use crate::common::*;

async_static! {
  test_cluster,
  Cluster,
  {
    Bot::cluster_inner(true)
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
  cluster: Cluster,
  instance_message_parser: Option<InstanceMessageParser>,
  user: discord::User,
  cache: InMemoryCache,
  db: Db,
}

impl Deref for Bot {
  type Target = Inner;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl Bot {
  pub(crate) fn main() -> Result<()> {
    logging::init();

    let runtime = runtime::init()?;

    runtime.block_on(async { Self::new(None).await?.run().await })?;

    Ok(())
  }

  pub(crate) fn is_test(&self) -> bool {
    self.instance_message_parser.is_some()
  }

  pub(crate) async fn run(self) -> Result<()> {
    info!("Starting run loop.");

    let mut events = self
      .cluster
      .some_events(EventTypeFlags::MESSAGE_CREATE | EventTypeFlags::REACTION_ADD);

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

    let channel_id = match &event {
      Event::MessageCreate(message_create) => message_create.channel_id,
      Event::ReactionAdd(reaction_add) => reaction_add.channel_id,
      _ => return Err(Error::UnexpectedEvent { event }),
    };

    let result = match event {
      Event::MessageCreate(message_create) => self.handle_message_create(*message_create).await,
      Event::ReactionAdd(reaction_add) => self.handle_reaction_add(*reaction_add).await,
      _ => Err(Error::UnexpectedEvent { event }),
    };

    if let Err(err) = result {
      self
        .create_message(
          channel_id,
          &format!(
            "Internal error: {}\n\nThis is a bug in Quwue.",
            err.message()
          ),
        )
        .await?;
    }

    Ok(())
  }

  async fn handle_reaction_add(&self, reaction_add: ReactionAdd) -> Result<()> {
    let ReactionAdd(reaction) = reaction_add;

    let user_id = reaction.user_id;

    let user = self.client().user(user_id).await?;

    let bot = if let Some(user) = user {
      user.bot
    } else {
      return Err(Error::UserUnavailable { user_id });
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
      .handle_response(bot, user, reaction.channel_id, response)
      .await?;

    Ok(())
  }

  async fn handle_message_create(&self, message: MessageCreate) -> Result<()> {
    let content = if let Some(parser) = &self.instance_message_parser {
      match parser.parse(message.content.as_str()) {
        Some(content) => content,
        None => return Ok(()),
      }
    } else {
      message.content.as_str()
    };

    fn extract_image_url(message: &MessageCreate) -> Option<String> {
      message.embeds.first()?.image.as_ref()?.url.to_owned()
    }

    let response = if let Some(text) = extract_image_url(&message) {
      Response::image(text.parse().context(error::EmbedImageUrlParse { text })?)
    } else {
      Response::message(content)
    };

    let user = self.db.user(message.author.id).await?;

    self
      .handle_response(message.author.bot, user, message.channel_id, response)
      .await?;

    Ok(())
  }

  async fn handle_response(
    &self,
    bot: bool,
    user: User,
    channel_id: ChannelId,
    response: Response,
  ) -> Result<()> {
    info!("Received response: {:?}", response);

    if user.discord_id == self.user.id {
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

    if !self.is_private_channel(channel_id).await {
      if self.is_test() {
        info!("Processing public channel message.");
      } else {
        return Err(Error::PublicResponse { response });
      }
    }

    let update = user.update(&response);

    let tx = self.db.prepare(user, update).await?;

    let prompt = tx.prompt();

    let prompt_message = self.create_message(channel_id, &prompt.text()).await?;

    for emoji in prompt.reactions() {
      let reaction_type = emoji.into();

      self
        .client()
        .create_reaction(channel_id, prompt_message.id, reaction_type)
        .await?;
    }

    tx.commit(prompt_message.id).await?;

    Ok(())
  }

  async fn is_private_channel(&self, id: ChannelId) -> bool {
    if let Some(private_channel) = self.cache.private_channel(id) {
      return matches!(private_channel.kind, ChannelType::Private);
    }

    let channel = self.client().channel(id).await.unwrap_or_default();

    match channel {
      Some(Channel::Private(_)) => true,
      Some(Channel::Group(_)) | Some(Channel::Guild(_)) | None => false,
    }
  }

  async fn create_message(&self, channel_id: ChannelId, content: &str) -> Result<Message> {
    let create_message = self.client().create_message(channel_id);

    let content = if let Some(parser) = &self.instance_message_parser {
      parser.prefix_message(content)
    } else {
      content.into()
    };

    Ok(create_message.content(content)?.await?)
  }

  #[cfg(test)]
  pub(crate) async fn test(instance_message_parser: InstanceMessageParser) -> Result<Self> {
    Self::new(Some(instance_message_parser)).await
  }

  fn client(&self) -> &Client {
    self.cluster.config().http_client()
  }

  async fn cluster_inner(test: bool) -> Result<Cluster> {
    let token = env::var("QUWUE_TOKEN").context(error::Token)?;

    let mut intents = Intents::DIRECT_MESSAGES | Intents::DIRECT_MESSAGE_REACTIONS;

    if test {
      intents |= Intents::GUILD_MESSAGES;
      intents |= Intents::GUILD_MESSAGE_REACTIONS;
    }

    let cluster = Cluster::new(token, intents).await?;

    cluster.up().await;

    cluster
      .some_events(EventTypeFlags::READY)
      .next()
      .await
      .expect("Did not receive ready event");

    Ok(cluster)
  }

  async fn cluster(test: bool) -> Result<Cluster> {
    if test {
      Ok(test_cluster::get().await.clone())
    } else {
      Self::cluster_inner(false).await
    }
  }

  async fn new(instance_message_parser: Option<InstanceMessageParser>) -> Result<Self> {
    let cluster = Self::cluster(instance_message_parser.is_some()).await?;

    let client = cluster.config().http_client();

    let user_id = client.current_user().await?.id;

    let user = client.user(user_id).await?.ok_or(Error::User)?;

    let cache = InMemoryCache::new();

    let db = Db::new().await?;

    let inner = Inner {
      cluster,
      cache,
      instance_message_parser,
      user,
      db,
    };

    Ok(Bot {
      inner: Arc::new(inner),
    })
  }
}
