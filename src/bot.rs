use crate::common::*;

async_static! {
  test_cluster,
  (Cluster, Arc<Mutex<Events>>),
  {
    Bot::initialize_cluster(true)
      .await
      .expect("Failed to initialize test cluster")
  }
}

#[derive(Clone, Debug)]
pub(crate) struct Bot {
  inner: Arc<Inner>,
}

#[derive(Debug)]
pub(crate) struct Inner {
  cache:   InMemoryCache,
  cluster: Cluster,
  db:      Db,
  events:  Arc<Mutex<Events>>,
  test_id: Option<TestId>,
  user:    twilight_model::user::User,
}

impl Deref for Bot {
  type Target = Inner;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl Bot {
  pub(crate) fn main() -> Result<()> {
    let arguments = Arguments::from_args();

    logging::init(arguments.log_dir.as_deref());

    let runtime = runtime::init()?;

    runtime.block_on(async { Self::new(&arguments.db_name, None).await?.run().await })?;

    Ok(())
  }

  pub(crate) fn is_test(&self) -> bool {
    self.test_id.is_some()
  }

  pub(crate) async fn run(self) -> Result<()> {
    info!("Starting run loop.");

    let mut events = self.events.lock().await;

    while let Some((shard_id, event)) = events.next().await {
      let clone = self.clone();
      let handle = tokio::spawn(async move {
        if let Err(err) = clone.handle_event(shard_id, event).await {
          if cfg!(test) {
            panic!("Error handling event: {}", err);
          } else {
            error!("Error handling event: {}", err);
          }
        }
      });

      if cfg!(test) {
        if let Err(err) = handle.await {
          if err.is_panic() {
            panic::resume_unwind(err.into_panic());
          }
        }
      }
    }

    Ok(())
  }

  async fn handle_event(self, _shard_id: u64, event: Event) -> Result<()> {
    info!("Quwue received event: {:?}", event.kind());

    self.cache.update(&event);

    let (channel_id, result) = match event {
      Event::MessageCreate(message_create) => (
        message_create.channel_id,
        self.handle_message_create(*message_create).await,
      ),
      Event::ReactionAdd(reaction_add) => (
        reaction_add.channel_id,
        self.handle_reaction_add(*reaction_add).await,
      ),
      _ => return Err(Error::UnexpectedEvent { event }),
    };

    if let Err(err) = result {
      eprintln!("Error handling event: {}", err);

      self
        .client()
        .create_message(channel_id)
        .content(&format!(
          "Internal error: {}\n\nThis is a bug in Quwue.",
          err.user_facing_message()
        ))?
        .exec()
        .await?;
    }

    Ok(())
  }

  async fn handle_reaction_add(&self, reaction_add: ReactionAdd) -> Result<()> {
    let ReactionAdd(reaction) = reaction_add;

    let bot = self
      .client()
      .user(reaction.user_id)
      .exec()
      .await?
      .model()
      .await?
      .bot;

    let user_id = if self.is_test() {
      let message = self
        .client()
        .message(reaction.channel_id, reaction.message_id)
        .exec()
        .await?
        .model()
        .await?;

      let test_message =
        TestMessage::parse(&message.content).expect("failed to parse reaction message");

      test_message.test_user_id().to_discord_user_id()
    } else {
      reaction.user_id
    };

    let user = self.db.user(user_id).await?;

    if user
      .prompt_message
      .map(|prompt_message| prompt_message.message_id != reaction.message_id)
      .unwrap_or_default()
    {
      return Ok(());
    }

    let response = match reaction.emoji {
      ReactionType::Unicode { name } => Response::unicode_reaction(name),
      ReactionType::Custom { id, .. } => Response::custom_reaction(id),
    };

    self
      .handle_response(bot, reaction.user_id, user, reaction.channel_id, response)
      .await?;

    Ok(())
  }

  async fn handle_message_create(&self, message: MessageCreate) -> Result<()> {
    let (sender_id, user_id, content) = if let Some(test_id) = &self.test_id {
      match test_id.filter(message.content.as_str()) {
        Some(test_message) => (
          message.author.id,
          test_message.test_user_id().to_discord_user_id(),
          test_message.text,
        ),
        None => return Ok(()),
      }
    } else {
      (
        message.author.id,
        message.author.id,
        message.content.clone(),
      )
    };

    let response = Response::message(content);

    let user = self.db.user(user_id).await?;

    self
      .handle_response(
        message.author.bot,
        sender_id,
        user,
        message.channel_id,
        response,
      )
      .await?;

    Ok(())
  }

  async fn handle_response(
    &self,
    bot: bool,
    sender: UserId,
    user: User,
    channel_id: ChannelId,
    response: Response,
  ) -> Result<()> {
    info!("Received response: {:?}", response);

    if sender == self.user.id {
      info!("Ignoring message from self.");
      return Ok(());
    }

    if bot {
      if self.is_test() {
        info!("Processing message from bot.");
      } else {
        return Err(Error::BotResponse { response });
      }
    }

    if !self.is_private_channel(channel_id).await? {
      if self.is_test() {
        info!("Processing public channel message.");
      } else {
        return Err(Error::PublicResponse { response });
      }
    }

    let update = user.update(&response);

    let user_id = user.discord_id;

    let tx = self.db.prepare(user_id, &update).await?;

    self.send_prompt(tx, channel_id, user_id).await?;

    if let Some(Action::AcceptCandidate { id: candidate_id }) = update.action {
      if let Some(tx) = self
        .db
        .prepare_interrupt_for_accept(user_id, candidate_id)
        .await?
      {
        let channel_id = if cfg!(test) {
          channel_id
        } else {
          self
            .client()
            .create_private_channel(candidate_id)
            .exec()
            .await?
            .model()
            .await?
            .id
        };
        self.send_prompt(tx, channel_id, candidate_id).await?;
      }
    }

    Ok(())
  }

  async fn send_prompt(
    &self,
    mut tx: UpdateTx<'_>,
    channel_id: ChannelId,
    recipient_id: UserId,
  ) -> Result<()> {
    let prompt = tx.prompt();

    let prompt_text = Db::prompt_text(&mut tx.inner_transaction(), prompt).await?;

    let avatar_url = if let Prompt::Candidate { id } | Prompt::Match { id } = prompt {
      let id = if cfg!(test) { self.user.id } else { id };
      self
        .client()
        .user(id)
        .exec()
        .await?
        .model()
        .await?
        .avatar
        .map(|hash| format!("https://cdn.discordapp.com/avatars/{}/{}.png", id, hash))
    } else {
      None
    };

    rate_limit::wait().await;
    let prompt_message = self
      .create_message(recipient_id, channel_id, &prompt_text, avatar_url)
      .await?;

    for emoji in prompt.reactions().iter().copied() {
      let reaction_type = emoji.into();

      rate_limit::wait().await;
      self
        .client()
        .create_reaction(channel_id, prompt_message.id, &reaction_type)
        .exec()
        .await?;
    }

    tx.commit(prompt_message.id).await?;

    Ok(())
  }

  async fn is_private_channel(&self, id: ChannelId) -> Result<bool> {
    if let Some(private_channel) = self.cache.private_channel(id) {
      return Ok(matches!(private_channel.kind, ChannelType::Private));
    }

    let channel = self.client().channel(id).exec().optional_model().await?;

    match channel {
      Some(Channel::Private(_)) => Ok(true),
      Some(Channel::Group(_) | Channel::Guild(_)) | None => Ok(false),
    }
  }

  async fn create_message(
    &self,
    user_id: UserId,
    channel_id: ChannelId,
    content: &str,
    image_url: Option<String>,
  ) -> Result<Message> {
    let mut create_message = self.client().create_message(channel_id);

    let content = self.test_id.as_ref().map_or_else(
      || content.into(),
      |test_id| test_id.prefix_message(user_id.0, content),
    );

    let mut embeds = Vec::new();
    if let Some(image_url) = image_url {
      embeds.push(
        EmbedBuilder::new()
          .image(ImageSource::url(image_url)?)
          .build()?,
      );
    }

    create_message = create_message.embeds(&embeds)?;

    Ok(
      create_message
        .content(&content)?
        .exec()
        .await?
        .model()
        .await?,
    )
  }

  #[cfg(test)]
  pub(crate) async fn new_test_instance(db_name: &str, test_id: TestId) -> Result<Self> {
    Self::new(db_name, Some(test_id)).await
  }

  pub(crate) fn client(&self) -> &Client {
    self.cluster.config().http_client()
  }

  async fn initialize_cluster(test: bool) -> Result<(Cluster, Arc<Mutex<Events>>)> {
    let token = env::var("QUWUE_TOKEN").context(error::Token)?;

    let mut intents = Intents::DIRECT_MESSAGES | Intents::DIRECT_MESSAGE_REACTIONS;

    if test {
      intents |= Intents::GUILD_MESSAGES;
      intents |= Intents::GUILD_MESSAGE_REACTIONS;
    }

    let (cluster, mut events) = Cluster::builder(token, intents)
      .event_types(
        EventTypeFlags::READY | EventTypeFlags::MESSAGE_CREATE | EventTypeFlags::REACTION_ADD,
      )
      .build()
      .await?;

    cluster.up().await;

    match events.next().await {
      Some((_, Event::Ready(_))) => {},
      event => return Err(Error::ClusterReady { event }),
    }

    Ok((cluster, Arc::new(Mutex::new(events))))
  }

  async fn new(db_name: &str, test_id: Option<TestId>) -> Result<Self> {
    let (cluster, events) = if test_id.is_some() {
      test_cluster::get().await.clone()
    } else {
      Self::initialize_cluster(false).await?
    };

    let client = cluster.config().http_client();

    let user_id = client.current_user().exec().await?.model().await?.id;

    let user = client.user(user_id).exec().await?.model().await?;

    let cache = InMemoryCache::new();

    let db = Db::connect(db_name).await?;

    let inner = Inner {
      cache,
      cluster,
      db,
      events,
      test_id,
      user,
    };

    Ok(Bot {
      inner: Arc::new(inner),
    })
  }

  #[cfg(test)]
  pub(crate) fn db(&self) -> &Db {
    &self.db
  }
}
