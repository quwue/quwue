use crate::common::*;

#[derive(Debug)]
pub(crate) struct Run {
  channel:            TextChannel,
  cluster:            Cluster,
  guild:              Guild,
  member:             Member,
  n:                  u64,
  user:               discord::User,
  mailboxes:          Arc<RwLock<BTreeMap<Instance, mpsc::UnboundedSender<(MessageId, Letter)>>>>,
  run_message_parser: RunMessageParser,
  rate_limiter:       Mutex<Instant>,
}

#[cfg(test)]
async_static! {
  test_run,
  Run,
  {
    logging::init();
    Run::init().await
  }
}

#[cfg(test)]
async_static! {
  dispatch,
  &'static Run,
  {
    let run = test_run::get().await;

    tokio::spawn(async move {
      run.dispatch().await;
    });

    run
  }
}

impl Run {
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

          if let Some((instance, content)) = self.run_message_parser.parse(&message.content) {
            if let Some(mailbox) = self.mailboxes.read().await.get(&instance) {
              mailbox
                .send((message.id, Letter::Message(content.into())))
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
            ReactionType::Custom { .. } =>
              panic!("Unexpected custom reaction: {:?}", reaction.emoji),
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

          if let Some((instance, _message)) = self.run_message_parser.parse(&message.content) {
            if let Some(mailbox) = self.mailboxes.read().await.get(&instance) {
              mailbox
                .send((message.id, Letter::Reaction(emoji)))
                .expect("message send failed");
            }
          }
        },
        _ => panic!("Unexpected event: {:?}", event.kind()),
      }
    }
  }

  pub(crate) async fn get() -> &'static Run {
    dispatch::get().await
  }

  async fn init() -> Run {
    info!("Initializing run instance…");

    let cluster = Run::initialize_cluster().await;

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

    const CHANNEL_NAME: &str = "testing";

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

    let n = member
      .nick
      .as_ref()
      .and_then(|nick| nick.parse::<u64>().map(|n| n + 1).ok())
      .unwrap_or(0);

    client
      .update_current_user_nick(guild.id, n.to_string())
      .await
      .unwrap();

    info!("Started test run {}.", n);

    client
      .create_message(channel.id)
      .content(format!("**Test Run {}**", n))
      .unwrap()
      .await
      .unwrap();

    let mailboxes = Arc::new(RwLock::new(BTreeMap::new()));

    let run_message_parser = RunMessageParser::new(n);

    info!("Run instance initialized.");

    Run {
      rate_limiter: Mutex::new(Instant::now()),
      channel,
      cluster,
      guild,
      mailboxes,
      run_message_parser,
      member,
      n,
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

  pub(crate) async fn register(
    &self,
    instance: &Instance,
  ) -> (
    mpsc::UnboundedReceiver<(MessageId, Letter)>,
    InstanceMessageParser,
  ) {
    let (snd, rcv) = mpsc::unbounded_channel();
    let mut mailboxes = self.mailboxes.write().await;
    if mailboxes.insert(instance.clone(), snd).is_some() {
      panic!("Second mailbox for instance {}!", instance);
    }
    (
      rcv,
      self
        .run_message_parser
        .instance_message_parser(instance.clone()),
    )
  }

  pub(crate) async fn wait(&self) {
    let mut rate_limiter = self.rate_limiter.lock().await;
    let now = Instant::now();

    if let Some(duration) = rate_limiter.checked_duration_since(now) {
      tokio::time::sleep(duration).await;
    }
    *rate_limiter = Instant::now() + Duration::from_secs(2);
  }

  pub(crate) async fn send(&self, instance: &Instance, msg: &str) {
    self.wait().await;
    let content = self.run_message_parser.prefix_message(instance, msg);
    self
      .client()
      .create_message(self.channel())
      .content(content)
      .unwrap()
      .await
      .unwrap();
  }

  pub(crate) async fn react(&self, id: MessageId, emoji: Emoji) {
    self.wait().await;
    self
      .client()
      .create_reaction(self.channel(), id, emoji.into())
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
