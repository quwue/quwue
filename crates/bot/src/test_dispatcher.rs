use crate::common::*;

type Channels = BTreeMap<TestUserId, mpsc::UnboundedSender<(MessageId, TestEvent)>>;

#[derive(Debug)]
pub(crate) struct TestDispatcher {
  channel:     TextChannel,
  cluster:     Cluster,
  guild:       Guild,
  member:      Member,
  test_run_id: TestRunId,
  user:        discord::User,
  channels:    Arc<RwLock<Channels>>,
}

#[cfg(test)]
async_static! {
  test_dispatcher_value,
  TestDispatcher,
  {
    logging::init();
    TestDispatcher::init().await
  }
}

#[cfg(test)]
async_static! {
  test_dispatcher_instance,
  &'static TestDispatcher,
  {
    let run = test_dispatcher_value::get().await;

    tokio::spawn(async move {
      run.dispatch().await;
    });

    run
  }
}

impl TestDispatcher {
  async fn dispatch(&self) {
    let mut events = self
      .cluster
      .some_events(EventTypeFlags::MESSAGE_CREATE | EventTypeFlags::REACTION_ADD);

    while let Some((_shard_id, event)) = events.next().await {
      info!("Received event: {:?}", event.kind());

      match event {
        Event::MessageCreate(message) => {
          if message.author.id == self.user.id {
            info!("Ignoring message from expect: {}", message.content);
            continue;
          }

          if let Some(test_message) = self.test_run_id.filter(&message.content) {
            if let Some(channel) = self.channels.read().await.get(&test_message.test_user_id()) {
              channel
                .send((message.id, TestEvent::Message(test_message.text)))
                .expect("message send failed");
            }
          }
        },
        Event::ReactionAdd(reaction) => {
          if reaction.user_id == self.user.id {
            info!("Ignoring reaction from expect: {:?}", reaction);
            continue;
          }

          let emoji = match &reaction.emoji {
            ReactionType::Custom { .. } => {
              panic!("Unexpected custom reaction: {:?}", reaction.emoji)
            },
            ReactionType::Unicode { name } =>
              if let Some(emoji) = Emoji::from_chars(&name) {
                emoji
              } else {
                panic!("Unrecognized reaction: {}", name);
              },
          };

          let message = self
            .client()
            .message(reaction.channel_id, reaction.message_id)
            .await
            .unwrap()
            .unwrap();

          if let Some(test_message) = self.test_run_id.filter(&message.content) {
            if let Some(channel) = self.channels.read().await.get(&test_message.test_user_id()) {
              channel
                .send((message.id, TestEvent::Reaction(emoji)))
                .expect("message send failed");
            }
          }
        },
        _ => panic!("Unexpected event: {:?}", event.kind()),
      }
    }
  }

  pub(crate) async fn get_instance() -> &'static TestDispatcher {
    test_dispatcher_instance::get().await
  }

  pub(crate) fn test_run_id(&self) -> TestRunId {
    self.test_run_id
  }

  async fn init() -> TestDispatcher {
    const CHANNEL_NAME: &str = "testing";

    info!("Initializing run instance…");

    let cluster = TestDispatcher::initialize_cluster().await;

    let client = cluster.config().http_client();

    let user_id = client.current_user().await.unwrap().id;

    let user = client.user(user_id).await.unwrap().unwrap();

    let guilds = client.current_user_guilds().await.unwrap();

    let guild = match guilds.len() {
      0 => panic!("Expect must be added to the testing guild."),
      1 => client.guild(guilds[0].id).await.unwrap().unwrap(),
      _ => panic!("Expect may not be in more than one guild."),
    };

    assert_eq!(
      guild.name, "Aesthetic Systems",
      "Unexpected testing guild name"
    );

    let channels = client.guild_channels(guild.id).await.unwrap();

    let mut testing_channels = channels
      .into_iter()
      .filter(|channel| channel.name() == CHANNEL_NAME)
      .collect::<Vec<GuildChannel>>();

    let channel = match testing_channels.len() {
      0 => client
        .create_guild_channel(guild.id, CHANNEL_NAME)
        .unwrap()
        .await
        .unwrap(),
      1 => testing_channels.remove(0),
      n => panic!("Found {} testing channels!", n),
    };

    let channel = match channel {
      GuildChannel::Text(channel) => channel,
      _ => panic!("Testing channel is not a text channel."),
    };

    let member = client
      .guild_member(guild.id, user.id)
      .await
      .unwrap()
      .unwrap();

    let test_run = member
      .nick
      .as_ref()
      .and_then(|nick| nick.parse::<u64>().map(|n| n + 1).ok())
      .unwrap_or(0);

    client
      .update_current_user_nick(guild.id, test_run.to_string())
      .await
      .unwrap();

    info!("Started test run {}.", test_run);

    client
      .create_message(channel.id)
      .content(format!("**Test Run {}**", test_run))
      .unwrap()
      .await
      .unwrap();

    let channels = Arc::new(RwLock::new(BTreeMap::new()));

    info!("TestDispatcher instance initialized.");

    Self {
      test_run_id: TestRunId::new(test_run),
      channel,
      cluster,
      guild,
      channels,
      member,
      user,
    }
  }

  async fn initialize_cluster() -> Cluster {
    #[derive(Deserialize, Debug)]
    struct Ratelimit {
      global:      bool,
      retry_after: f64,
      message:     String,
    }

    let token = expect_var("EXPECT_TOKEN");

    let cluster = loop {
      match Cluster::new(
        &token,
        Intents::GUILD_MESSAGES | Intents::GUILD_MESSAGE_REACTIONS,
      )
      .await
      {
        Ok(cluster) => break cluster,
        Err(ClusterStartError::RetrievingGatewayInfo {
          source: HttpError::Response { body, status, .. },
        }) if status == StatusCode::TOO_MANY_REQUESTS => {
          let body = String::from_utf8_lossy(&body);
          match serde_json::from_str::<Ratelimit>(&body) {
            Err(serde_error) => panic!(
              "Failed to deserialize response body: {}\n{}",
              serde_error, body,
            ),
            Ok(ratelimit) =>
              if ratelimit.global {
                panic!("Ratelimited globally: {:?}", ratelimit);
              } else {
                let duration = Duration::from_secs_f64(ratelimit.retry_after);
                info!("Retrying after {} seconds…", duration.as_secs());
                time::sleep(duration).await;
              },
          }
        },
        Err(other) => panic!("Received unexpected cluster start error: {}", other),
      };
    };

    cluster.up().await;

    cluster
      .some_events(EventTypeFlags::READY)
      .next()
      .await
      .unwrap();

    cluster
  }

  pub(crate) async fn register_test_user(
    &self,
    test_user_id: &TestUserId,
  ) -> mpsc::UnboundedReceiver<(MessageId, TestEvent)> {
    let (tx, rx) = mpsc::unbounded_channel();
    let mut channel = self.channels.write().await;
    if channel.insert(test_user_id.clone(), tx).is_some() {
      panic!("Second channel for test user {}!", test_user_id);
    }
    rx
  }

  pub(crate) async fn send_message(&self, test_user_id: &TestUserId, msg: &str) {
    rate_limit::wait().await;
    let content = self.test_run_id.prefix_message(test_user_id, msg);
    self
      .client()
      .create_message(self.channel())
      .content(content)
      .unwrap()
      .await
      .unwrap();
  }

  pub(crate) async fn send_reaction(&self, id: MessageId, emoji: Emoji) {
    rate_limit::wait().await;
    self
      .client()
      .create_reaction(self.channel(), id, emoji.into())
      .await
      .unwrap();
  }

  pub(crate) async fn send_attachment(
    &self,
    test_user_id: &TestUserId,
    filename: &str,
    data: Vec<u8>,
  ) {
    rate_limit::wait().await;
    let content = self.test_run_id.prefix_message(test_user_id, "");
    self
      .client()
      .create_message(self.channel())
      .content(content)
      .unwrap()
      .attachment(filename, data)
      .await
      .unwrap();
  }

  pub(crate) fn client(&self) -> &Client {
    self.cluster.config().http_client()
  }

  pub(crate) fn channel(&self) -> ChannelId {
    self.channel.id
  }
}
