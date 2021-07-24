use crate::common::*;

#[derive(Debug)]
pub struct Db {
  pool: SqlitePool,
}

macro_rules! load_user {
  {$user:expr} => {
    {
      let user = $user;

      let prompt = match (user.prompt, user.prompt_payload) {
        (Some(prompt), prompt_payload) => Some(Prompt::load((prompt, prompt_payload))?),
        (None, Some(_)) => todo!(),
        (None, None) => None
      };

      // user.prompt.map(Prompt::load).transpose()?;

      let message_id = user
        .prompt_message_id
        .map(MessageId::load)
        .transpose()
        .unwrap_infallible();

      let prompt_message = match (prompt, message_id) {
        (Some(prompt), Some(message_id)) => Some(PromptMessage {
          prompt,
          message_id
        }),
        (None, None) => None,
        (prompt, message_id) => return Err(Error::PromptMessageLoad {
          prompt,
          message_id
        }),
      };

      let profile_image_url =  if let Some(text) = user.profile_image_url {
        Some(text.parse().context(error::UrlLoad { text })?)
      } else {
        None
      };

      User {
        id:             u64::load(user.id).unwrap_infallible(),
        discord_id:     UserId::load(user.discord_id).unwrap_infallible(),
        welcomed:       user.welcomed,
        bio:            user.bio,
        profile_image_url,
        prompt_message,
      }
    }
  }
}

impl Db {
  pub async fn connect(path: &Path) -> Result<Self> {
    let url = db_url::db_url(path).ok_or_else(|| Error::PathUnicodeDecode {
      path: path.to_owned(),
    })?;

    Sqlite::create_database(&url).await.unwrap();

    let pool = SqlitePool::connect(&url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(Self { pool })
  }

  pub async fn user(&self, discord_id: UserId) -> Result<User> {
    let discord_id = discord_id.store();

    let row = sqlx::query!("SELECT * FROM users WHERE discord_id = ?", discord_id)
      .fetch_optional(&self.pool)
      .await?;

    if let Some(user) = row {
      return Ok(load_user!(user));
    }

    let mut tx = self.pool.begin().await?;

    sqlx::query!(
      "INSERT OR IGNORE INTO users(discord_id) VALUES(?)",
      discord_id,
    )
    .execute(&mut tx)
    .await?;

    let user = sqlx::query!("SELECT * FROM users WHERE discord_id = ?", discord_id)
      .fetch_one(&mut tx)
      .await?;

    tx.commit().await?;

    Ok(load_user!(user))
  }

  async fn candidate<'a>(tx: &mut Transaction<'a>, discord_id: UserId) -> Result<Option<UserId>> {
    let discord_id = discord_id.store();

    let row = sqlx::query!(
      "SELECT
        discord_id
      FROM
        users
      WHERE
        welcomed == TRUE
        AND
        bio IS NOT NULL
        AND
        profile_image_url IS NOT NULL
        AND
        discord_id != ?
        AND
        NOT EXISTS (
          SELECT * FROM responses
          WHERE discord_id = ? AND candidate_id = users.discord_id
        )
        AND
        NOT EXISTS (
          SELECT * FROM responses
          WHERE discord_id = users.discord_id AND candidate_id = ? AND NOT response
        )
      LIMIT 1",
      discord_id,
      discord_id,
      discord_id
    )
    .fetch_optional(tx)
    .await?;

    Ok(row.map(|row| UserId::load(row.discord_id).unwrap_infallible()))
  }

  async fn get_match<'a>(tx: &mut Transaction<'a>, discord_id: UserId) -> Result<Option<UserId>> {
    let discord_id = discord_id.store();

    let row = sqlx::query!(
      "SELECT
        candidate_id
      FROM
        responses as outer
      WHERE
        discord_id = ?
        AND
        response
        AND
        EXISTS (
          SELECT * FROM responses
          WHERE
            discord_id = outer.candidate_id
            AND
            candidate_id = outer.discord_id
            AND
            response
        )
      LIMIT 1",
      discord_id,
    )
    .fetch_optional(tx)
    .await?;

    Ok(row.map(|row| UserId::load(row.candidate_id).unwrap_infallible()))
  }

  pub async fn prepare<'a>(&'a self, user_id: UserId, update: &Update) -> Result<UpdateTx<'a>> {
    let mut tx = self.pool.begin().await?;

    if let Some(action) = &update.action {
      use Action::*;
      match action {
        Welcome => Self::welcome(&mut tx, user_id).await?,
        SetBio { text } => Self::set_bio(&mut tx, user_id, &text).await?,
        SetProfileImage { url } => Self::set_profile_image(&mut tx, user_id, &url).await?,
        AcceptCandidate { id } => Self::respond_to_candidate(&mut tx, user_id, *id, true).await?,
        RejectCandidate { id } => Self::respond_to_candidate(&mut tx, user_id, *id, false).await?,
      }
    }

    let mut prompt = update.prompt;

    if prompt.quiescent() {
      if let Some(id) = Self::get_match(&mut tx, user_id).await? {
        prompt = Prompt::Match { id };
      } else if let Some(id) = Self::candidate(&mut tx, user_id).await? {
        prompt = Prompt::Candidate { id };
      }
    };

    let update_tx = UpdateTx {
      prompt,
      tx,
      user_id,
    };

    Ok(update_tx)
  }

  pub async fn prepare_interrupt_for_accept<'a>(
    &'a self,
    user_id: UserId,
    candidate_id: UserId,
  ) -> Result<Option<UpdateTx<'a>>> {
    let mut tx = self.pool.begin().await?;

    let row = {
      let user_id = user_id.store();
      let candidate_id = candidate_id.store();

      sqlx::query!(
        "SELECT
        response
      FROM
        responses
      WHERE
        discord_id = ? AND candidate_id = ?
      LIMIT 1",
        candidate_id,
        user_id,
      )
      .fetch_optional(&mut tx)
      .await?
      .map(|row| row.response)
    };

    let prompt = match row {
      Some(true) => Prompt::Match { id: user_id },
      Some(false) => return Ok(None),
      None => Prompt::Candidate { id: user_id },
    };

    let update_tx = UpdateTx {
      user_id: candidate_id,
      prompt,
      tx,
    };

    Ok(Some(update_tx))
  }

  pub(crate) async fn commit<'a>(
    mut tx: Transaction<'a>,
    discord_id: UserId,
    prompt_message: PromptMessage,
  ) -> Result<()> {
    let discord_id = discord_id.store();
    let (prompt, prompt_payload) = prompt_message.prompt.store();
    let (prompt, prompt_payload) = (Some(prompt), Some(prompt_payload));
    let prompt_message_id = Some(prompt_message.message_id.store());

    sqlx::query!(
      "UPDATE
        users
      SET
        prompt = ?,
        prompt_payload = ?,
        prompt_message_id = ?
      WHERE discord_id = ?",
      prompt,
      prompt_payload,
      prompt_message_id,
      discord_id
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(())
  }

  async fn welcome(tx: &mut Transaction<'_>, discord_id: UserId) -> Result<()> {
    let discord_id = discord_id.store();

    sqlx::query!(
      "UPDATE users SET welcomed = true WHERE discord_id = ?",
      discord_id
    )
    .execute(tx)
    .await?;

    Ok(())
  }

  async fn set_bio(tx: &mut Transaction<'_>, discord_id: UserId, text: &str) -> Result<()> {
    let discord_id = discord_id.store();

    sqlx::query!(
      "UPDATE users SET bio = ? WHERE discord_id = ?",
      text,
      discord_id
    )
    .execute(tx)
    .await?;

    Ok(())
  }

  async fn set_profile_image(
    tx: &mut Transaction<'_>,
    discord_id: UserId,
    url: &Url,
  ) -> Result<()> {
    let discord_id = discord_id.store();
    let url = url.as_str();

    sqlx::query!(
      "UPDATE users SET profile_image_url = ? where discord_id = ?",
      url,
      discord_id
    )
    .execute(tx)
    .await?;

    Ok(())
  }

  #[cfg(test)]
  async fn user_count(&self) -> Result<u64> {
    #[allow(clippy::cast_sign_loss)]
    Ok(
      sqlx::query!("SELECT COUNT(*) as count FROM users")
        .fetch_one(&self.pool)
        .await?
        .count as u64,
    )
  }

  pub async fn prompt_text(tx: &mut Transaction<'_>, prompt: Prompt) -> Result<String> {
    use Prompt::*;

    let text = match prompt {
      Welcome => format!(
        concat!(
          "Hi!\n",
          "Quwue is a bot that matches you with other Discord users.\n",
          "Your Discord tag will only be revealed to matches.\n",
          "To start, you'll need to set up your profile.\n",
          "React with {} or type `ok` to continue.",
        ),
        Emoji::ThumbsUp.markup()
      ),
      Quiescent => "You've seen all available matches. We'll message you when we have new matches \
                    to show you!"
        .into(),
      Candidate { id } => {
        format!("New potential match:\n{}", Self::bio(tx, id).await?)
      }
      Bio => "Please enter a bio to show to other users.".into(),
      ProfileImage => "Please upload a profile photo.".into(),
      Match { id } => format!(
        "You matched with <@{}>:\n{}\nSend them a message!",
        id,
        Self::bio(tx, id).await?,
      ),
    };

    Ok(text)
  }

  async fn bio(tx: &mut Transaction<'_>, id: UserId) -> Result<String> {
    let id_storage = id.store();

    let row = sqlx::query!("SELECT bio from users where discord_id = ?", id_storage)
      .fetch_optional(tx)
      .await?;

    row
      .ok_or(Error::UserUnknown { id })?
      .bio
      .ok_or(Error::UserMissingBio { id })
  }

  async fn respond_to_candidate(
    tx: &mut Transaction<'_>,
    user_id: UserId,
    candidate_id: UserId,
    response: bool,
  ) -> Result<()> {
    let user_id = user_id.store();
    let candidate_id = candidate_id.store();

    sqlx::query!(
      "INSERT OR REPLACE INTO responses
        (discord_id, candidate_id, response)
      VALUES
        (?, ?, ?)",
      user_id,
      candidate_id,
      response
    )
    .execute(tx)
    .await?;

    Ok(())
  }

  pub async fn prompt_text_outside_update_transaction(&self, prompt: Prompt) -> String {
    let mut tx = self.pool.begin().await.unwrap();
    Db::prompt_text(&mut tx, prompt).await.unwrap()
  }

  #[cfg(test)]
  async fn create_profile(&self, id: UserId) {
    self.user(id).await.unwrap();

    let update = Update {
      action: Some(Action::Welcome),
      prompt: Prompt::Bio,
    };

    let tx = self.prepare(id, &update).await.unwrap();

    tx.commit(MessageId(200)).await.unwrap();

    let update = Update {
      action: Some(Action::SetBio {
        text: format!("User {}'s bio!", id),
      }),
      prompt: Prompt::ProfileImage,
    };

    let tx = self.prepare(id, &update).await.unwrap();

    tx.commit(MessageId(200)).await.unwrap();

    let update = Update {
      action: Some(Action::SetProfileImage {
        url: format!("https://foo.example/user-{}.png", id)
          .parse()
          .unwrap(),
      }),
      prompt: Prompt::Quiescent,
    };

    let tx = self.prepare(id, &update).await.unwrap();

    tx.commit(MessageId(200)).await.unwrap();
  }

  #[cfg(test)]
  async fn response(&self, user: UserId, candidate: UserId) -> bool {
    let user = user.store();
    let candidate = candidate.store();
    let row = sqlx::query!(
      "SELECT response FROM responses WHERE discord_id = ? AND candidate_id = ?",
      user,
      candidate
    )
    .fetch_one(&self.pool)
    .await
    .unwrap();

    row.response
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct TestContext {
    tmpdir: TempDir,
    db: Db,
    db_path: PathBuf,
  }

  impl TestContext {
    async fn new() -> Self {
      let tmpdir = tempdir().unwrap();

      let db_path = tmpdir.path().join("db.sqlite");

      let db = Db::connect(&db_path).await.unwrap();

      TestContext {
        tmpdir,
        db,
        db_path,
      }
    }
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn on_disk_database_is_persistant() {
    let TestContext {
      tmpdir: _tmpdir,
      db,
      db_path,
    } = TestContext::new().await;

    assert_eq!(db.user_count().await.unwrap(), 0);

    let a = UserId(100);
    db.create_profile(a).await;

    assert_eq!(db.user_count().await.unwrap(), 1);

    drop(db);

    let db = Db::connect(&db_path).await.unwrap();

    assert_eq!(db.user_count().await.unwrap(), 1);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn create_user() {
    let context = TestContext::new().await;

    let discord_id = UserId(100);

    assert_eq!(context.db.user_count().await.unwrap(), 0);

    let have = context.db.user(discord_id).await.unwrap();
    let want = User {
      id: 1,
      prompt_message: None,
      welcomed: false,
      bio: None,
      profile_image_url: None,
      discord_id,
    };
    assert_eq!(have, want);

    assert_eq!(context.db.user_count().await.unwrap(), 1);

    let have = context.db.user(discord_id).await.unwrap();
    assert_eq!(have, want);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn welcome() {
    let context = TestContext::new().await;

    let discord_id = UserId(100);
    let message_id = MessageId(200);

    let have = context.db.user(discord_id).await.unwrap();
    let want = User {
      id: 1,
      prompt_message: None,
      welcomed: false,
      bio: None,
      profile_image_url: None,
      discord_id,
    };
    assert_eq!(have, want);

    let prompt_message = PromptMessage {
      prompt: Prompt::Welcome,
      message_id,
    };

    let update = Update {
      action: Some(Action::Welcome),
      prompt: Prompt::Welcome,
    };

    let tx = context.db.prepare(have.discord_id, &update).await.unwrap();

    tx.commit(message_id).await.unwrap();

    let have = context.db.user(discord_id).await.unwrap();
    let want = User {
      id: 1,
      welcomed: true,
      prompt_message: Some(prompt_message),
      bio: None,
      profile_image_url: None,
      discord_id,
    };
    assert_eq!(have, want);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn set_bio() {
    let context = TestContext::new().await;

    let discord_id = UserId(100);
    let message_id = MessageId(200);

    let have = context.db.user(discord_id).await.unwrap();
    let want = User {
      id: 1,
      prompt_message: None,
      welcomed: false,
      bio: None,
      profile_image_url: None,
      discord_id,
    };
    assert_eq!(have, want);

    let prompt_message = PromptMessage {
      prompt: Prompt::Bio,
      message_id,
    };

    let update = Update {
      action: Some(Action::SetBio {
        text: "bio!".to_owned(),
      }),
      prompt: Prompt::Bio,
    };

    let tx = context.db.prepare(have.discord_id, &update).await.unwrap();

    tx.commit(message_id).await.unwrap();

    let have = context.db.user(discord_id).await.unwrap();
    let want = User {
      id: 1,
      welcomed: false,
      prompt_message: Some(prompt_message),
      bio: Some("bio!".to_owned()),
      profile_image_url: None,
      discord_id,
    };
    assert_eq!(have, want);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn set_profile_image_url() {
    let context = TestContext::new().await;

    let discord_id = UserId(100);
    let message_id = MessageId(200);

    let have = context.db.user(discord_id).await.unwrap();
    let want = User {
      id: 1,
      prompt_message: None,
      welcomed: false,
      bio: None,
      profile_image_url: None,
      discord_id,
    };
    assert_eq!(have, want);

    let prompt_message = PromptMessage {
      prompt: Prompt::ProfileImage,
      message_id,
    };

    let update = Update {
      action: Some(Action::SetProfileImage {
        url: "https://www.google.com".parse().unwrap(),
      }),
      prompt: Prompt::ProfileImage,
    };

    let tx = context.db.prepare(have.discord_id, &update).await.unwrap();

    tx.commit(message_id).await.unwrap();

    let have = context.db.user(discord_id).await.unwrap();
    let want = User {
      id: 1,
      welcomed: false,
      prompt_message: Some(prompt_message),
      bio: None,
      profile_image_url: Some("https://www.google.com".parse().unwrap()),
      discord_id,
    };
    assert_eq!(have, want);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn expect_candidate() {
    let context = TestContext::new().await;

    let a = UserId(100);
    let b = UserId(101);

    context.db.create_profile(a).await;
    context.db.create_profile(b).await;

    let update = Update {
      action: None,
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Candidate { id: b })
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn filter_out_accepted_candidates() {
    let context = TestContext::new().await;

    let a = UserId(100);
    let b = UserId(101);

    context.db.create_profile(a).await;
    context.db.create_profile(b).await;

    let update = Update {
      action: None,
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Candidate { id: b });

    tx.commit(MessageId(200)).await.unwrap();

    let update = Update {
      action: Some(Action::AcceptCandidate { id: b }),
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn filter_out_rejected_candidates() {
    let context = TestContext::new().await;

    let a = UserId(100);
    let b = UserId(101);

    context.db.create_profile(a).await;
    context.db.create_profile(b).await;

    let update = Update {
      action: None,
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Candidate { id: b });

    tx.commit(MessageId(200)).await.unwrap();

    let update = Update {
      action: Some(Action::RejectCandidate { id: b }),
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn filter_out_candidates_that_have_rejected_user() {
    let context = TestContext::new().await;

    let a = UserId(100);
    let b = UserId(101);

    context.db.create_profile(a).await;
    context.db.create_profile(b).await;

    let update = Update {
      action: None,
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Candidate { id: b });

    tx.commit(MessageId(200)).await.unwrap();

    let update = Update {
      action: Some(Action::RejectCandidate { id: b }),
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);

    tx.commit(MessageId(201)).await.unwrap();

    let update = Update {
      action: None,
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(b, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn dont_filter_candidates_that_have_accepted_user() {
    let context = TestContext::new().await;

    let a = UserId(100);
    let b = UserId(101);

    context.db.create_profile(a).await;
    context.db.create_profile(b).await;

    let update = Update {
      action: None,
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Candidate { id: b });

    tx.commit(MessageId(200)).await.unwrap();

    let update = Update {
      action: Some(Action::AcceptCandidate { id: b }),
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);

    tx.commit(MessageId(201)).await.unwrap();

    let update = Update {
      action: None,
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(b, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Candidate { id: a });
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn allow_multiple_responses() {
    let context = TestContext::new().await;

    let a = UserId(100);
    let b = UserId(101);

    context.db.create_profile(a).await;
    context.db.create_profile(b).await;

    let update = Update {
      action: None,
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Candidate { id: b });

    tx.commit(MessageId(200)).await.unwrap();

    let update = Update {
      action: Some(Action::AcceptCandidate { id: b }),
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);

    tx.commit(MessageId(201)).await.unwrap();

    assert!(context.db.response(a, b).await);

    let update = Update {
      action: Some(Action::RejectCandidate { id: b }),
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);

    tx.commit(MessageId(201)).await.unwrap();

    assert!(!context.db.response(a, b).await);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn show_match_prompt_after_mutual_acceptance() {
    let context = TestContext::new().await;

    let a = UserId(100);
    let b = UserId(101);

    context.db.create_profile(a).await;
    context.db.create_profile(b).await;

    let update = Update {
      action: Some(Action::AcceptCandidate { id: b }),
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);

    tx.commit(MessageId(201)).await.unwrap();

    let update = Update {
      action: Some(Action::AcceptCandidate { id: a }),
      prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(b, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Match { id: a });

    tx.commit(MessageId(201)).await.unwrap();
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn inserting_responses_from_non_existant_users_is_an_error() {
    let context = TestContext::new().await;

    let a = UserId(100);
    context.db.create_profile(a).await;

    let mut tx = context.db.pool.begin().await.unwrap();

    let error = sqlx::query!(
      "INSERT INTO responses
        (discord_id, candidate_id, response)
      VALUES
        (1, 100, 1)",
    )
    .execute(&mut tx)
    .await
    .unwrap_err();

    guard_unwrap!(let sqlx::Error::Database(error) = error);

    assert_eq!(error.message(), "FOREIGN KEY constraint failed");
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn inserting_responses_to_non_existant_users_is_an_error() {
    let context = TestContext::new().await;

    let a = UserId(100);
    context.db.create_profile(a).await;

    let mut tx = context.db.pool.begin().await.unwrap();

    let error = sqlx::query!(
      "INSERT INTO responses
        (discord_id, candidate_id, response)
      VALUES
        (100, 1, 1)",
    )
    .execute(&mut tx)
    .await
    .unwrap_err();

    guard_unwrap!(let sqlx::Error::Database(error) = error);

    assert_eq!(error.message(), "FOREIGN KEY constraint failed");
  }
}
