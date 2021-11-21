use crate::common::*;

#[derive(Debug)]
pub struct Db {
  pool: PgPool,
}

impl Db {
  pub async fn connect(name: &str) -> Result<Self> {
    let url = db_url::db_url(name);

    if !sqlx::Postgres::database_exists(&url).await? {
      Postgres::create_database(&url).await.unwrap();
    }

    let options = sqlx::postgres::PgConnectOptions::from_str(&url)?;

    let pool = PgPool::connect_with(options).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(Self { pool })
  }

  async fn load_user<'a>(tx: &mut Transaction<'a>, discord_id: UserId) -> Result<Option<User>> {
    let discord_id = discord_id.store();

    let row = sqlx::query!("SELECT * FROM users WHERE discord_id = $1", discord_id)
      .fetch_optional(&mut *tx)
      .await?;

    if let Some(user) = row {
      let prompt = sqlx::query!(
        "SELECT * FROM prompts where recipient_discord_id = $1",
        discord_id,
      )
      .fetch_optional(tx)
      .await?;

      let prompt_message = match prompt {
        Some(row) => Some(PromptMessage {
          prompt:     Prompt::load((row.discriminant, row.payload))?,
          message_id: MessageId::load(row.message_id).unwrap_infallible(),
        }),
        None => None,
      };

      return Ok(Some(User {
        id: u64::load(user.id).unwrap_infallible(),
        discord_id: UserId::load(user.discord_id).unwrap_infallible(),
        welcomed: user.welcomed,
        bio: user.bio,
        prompt_message,
      }));
    }

    Ok(None)
  }

  pub async fn user(&self, discord_id: UserId) -> Result<User> {
    let mut tx = self.pool.begin().await?;

    if let Some(user) = Self::load_user(&mut tx, discord_id).await? {
      return Ok(user);
    }

    {
      let discord_id = discord_id.store();

      sqlx::query!("INSERT INTO users(discord_id) VALUES($1)", discord_id)
        .execute(&mut tx)
        .await?;
    };

    let user = Self::load_user(&mut tx, discord_id)
      .await?
      .ok_or_else(|| Error::Internal {
        message: "Load user returned None after insertion.".to_owned(),
      })?;

    tx.commit().await?;

    Ok(user)
  }

  async fn get_candidate<'a>(
    tx: &mut Transaction<'a>,
    discord_id: UserId,
  ) -> Result<Option<UserId>> {
    let discord_id = discord_id.store();

    let quiescent_discriminant = PromptDiscriminant::Quiescent.store();

    // This should be a single SELECT with an OUTER JOIN, instead of two SELECTS,
    // but that seems to break the sqlx query parser:
    // https://github.com/launchbadge/sqlx/issues/1249
    let candidate = sqlx::query!(
      "SELECT
        discord_id
      FROM
        users AS potential_candidate
      WHERE
        welcomed = TRUE
        AND
        bio IS NOT NULL
        AND
        discord_id != $1
        AND
        NOT EXISTS (
          SELECT * FROM responses
          WHERE discord_id = $2 AND candidate_id = potential_candidate.discord_id
        )
        AND
        EXISTS (
          SELECT * FROM responses
          WHERE discord_id = potential_candidate.discord_id AND candidate_id = $3 AND response
        )
        AND
        EXISTS (
          SELECT * FROM prompts
          WHERE
            recipient_discord_id = potential_candidate.discord_id AND discriminant = $4
        )
      LIMIT 1",
      discord_id,
      discord_id,
      discord_id,
      quiescent_discriminant,
    )
    .fetch_optional(&mut *tx)
    .await?
    .map(|row| UserId::load(row.discord_id).unwrap_infallible());

    if let Some(candidate) = candidate {
      return Ok(Some(candidate));
    }

    let candidate = sqlx::query!(
      "SELECT
        discord_id
      FROM
        users AS potential_candidate
      WHERE
        welcomed = TRUE
        AND
        bio IS NOT NULL
        AND
        discord_id != $1
        AND
        NOT EXISTS (
          SELECT * FROM responses
          WHERE discord_id = $2 AND candidate_id = potential_candidate.discord_id
        )
        AND
        NOT EXISTS (
          SELECT * FROM responses
          WHERE discord_id = potential_candidate.discord_id AND candidate_id = $3 AND NOT response
        )
        AND
        EXISTS (
          SELECT * FROM prompts
          WHERE
            recipient_discord_id = potential_candidate.discord_id AND discriminant = $4
        )
      LIMIT 1",
      discord_id,
      discord_id,
      discord_id,
      quiescent_discriminant,
    )
    .fetch_optional(tx)
    .await?
    .map(|row| UserId::load(row.discord_id).unwrap_infallible());

    Ok(candidate)
  }

  async fn get_match<'a>(tx: &mut Transaction<'a>, discord_id: UserId) -> Result<Option<UserId>> {
    let discord_id = discord_id.store();

    let row = sqlx::query!(
      "SELECT
        candidate_id
      FROM
        responses AS outer_responses
      WHERE
        discord_id = $1
        AND
        response
        AND
        NOT dismissed
        AND
        EXISTS (
          SELECT * FROM responses
          WHERE
            discord_id = outer_responses.candidate_id
            AND
            candidate_id = outer_responses.discord_id
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
        SetBio { text } => Self::set_bio(&mut tx, user_id, text).await?,
        AcceptCandidate { id } => Self::respond_to_candidate(&mut tx, user_id, *id, true).await?,
        DeclineCandidate { id } => Self::respond_to_candidate(&mut tx, user_id, *id, false).await?,
        DismissMatch { id } => Self::dismiss_match(&mut tx, user_id, *id).await?,
      }
    }

    let mut next_prompt = update.next_prompt;

    if next_prompt.quiescent() {
      if let Some(id) = Self::get_match(&mut tx, user_id).await? {
        next_prompt = Prompt::Match { id };
      } else if let Some(id) = Self::get_candidate(&mut tx, user_id).await? {
        next_prompt = Prompt::Candidate { id };
      }
    };

    let update_tx = UpdateTx {
      prompt: next_prompt,
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

    let response = {
      let user_id = user_id.store();
      let candidate_id = candidate_id.store();

      sqlx::query!(
        "SELECT
          response
        FROM
          responses
        WHERE
          discord_id = $1 AND candidate_id = $2
        LIMIT 1",
        candidate_id,
        user_id,
      )
      .fetch_optional(&mut tx)
      .await?
      .map(|row| row.response)
    };

    let prompt = match response {
      Some(true) => Prompt::Match { id: user_id },
      Some(false) => return Ok(None),
      None => Prompt::Candidate { id: user_id },
    };

    {
      let candidate_id = candidate_id.store();

      let discriminant = sqlx::query!(
        "SELECT
          discriminant
        FROM
          prompts
        WHERE
          recipient_discord_id = $1
        LIMIT 1",
        candidate_id,
      )
      .fetch_optional(&mut tx)
      .await?
      .map(|row| row.discriminant);

      if let Some(discriminant) = discriminant {
        if prompt.cannot_interrupt(PromptDiscriminant::load(discriminant)?) {
          return Ok(None);
        }
      }
    }

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
    let (discriminant, payload) = prompt_message.prompt.store();
    let message_id = prompt_message.message_id.store();

    sqlx::query!(
      "INSERT INTO prompts
        (discriminant, payload, message_id, recipient_discord_id)
      VALUES
        ($1, $2, $3, $4)
      ON CONFLICT (recipient_discord_id) DO UPDATE SET
        discriminant = $1,
        payload = $2,
        message_id = $3,
        recipient_discord_id = $4
      ",
      discriminant,
      payload,
      message_id,
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
      "UPDATE users SET welcomed = TRUE WHERE discord_id = $1",
      discord_id
    )
    .execute(tx)
    .await?;

    Ok(())
  }

  async fn set_bio(tx: &mut Transaction<'_>, discord_id: UserId, text: &str) -> Result<()> {
    let discord_id = discord_id.store();

    sqlx::query!(
      "UPDATE users SET bio = $1 WHERE discord_id = $2",
      text,
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
        .count
        .unwrap_or(0) as u64,
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
      },
      Bio => "Please enter a bio to show to other users.".into(),
      Match { id } => format!(
        concat!(
          "You matched with <@{}>:\n{}\nSend them a message!\n",
          "React with {} or type `ok` to continue.",
        ),
        id,
        Self::bio(tx, id).await?,
        Emoji::ThumbsUp.markup()
      ),
    };

    Ok(text)
  }

  async fn bio(tx: &mut Transaction<'_>, id: UserId) -> Result<String> {
    let id_storage = id.store();

    let row = sqlx::query!("SELECT bio from users where discord_id = $1", id_storage)
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
      "INSERT INTO responses
        (discord_id, candidate_id, response, dismissed)
      VALUES
        ($1, $2, $3, FALSE)
      ON CONFLICT (discord_id, candidate_id) DO UPDATE SET
        discord_id = $1,
        candidate_id = $2,
        response = $3,
        dismissed = FALSE
      ",
      user_id,
      candidate_id,
      response
    )
    .execute(tx)
    .await?;

    Ok(())
  }

  async fn dismiss_match(
    tx: &mut Transaction<'_>,
    user_id: UserId,
    match_id: UserId,
  ) -> Result<()> {
    let user_id = user_id.store();
    let match_id = match_id.store();

    sqlx::query!(
      "UPDATE responses SET dismissed = TRUE WHERE discord_id = $1 AND candidate_id = $2",
      user_id,
      match_id
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
  async fn create_user(&self, expected_prompt: Prompt) -> UserId {
    let id = sqlx::query!(
      "SELECT
        discord_id
      FROM
        users
      ORDER BY
        discord_id DESC
      "
    )
    .fetch_optional(&self.pool)
    .await
    .unwrap()
    .map_or(0, |row| row.discord_id + 1);

    let id = UserId(u64::load(id).unwrap());

    self.user(id).await.unwrap();

    let update = Update {
      action:      Some(Action::Welcome),
      next_prompt: Prompt::Bio,
    };

    let tx = self.prepare(id, &update).await.unwrap();

    tx.commit(MessageId(200)).await.unwrap();

    let update = Update {
      action:      Some(Action::SetBio {
        text: format!("User {}'s bio!", id),
      }),
      next_prompt: Prompt::Quiescent,
    };

    let tx = self.prepare(id, &update).await.unwrap();

    assert_eq!(tx.prompt(), expected_prompt);

    tx.commit(MessageId(200)).await.unwrap();

    id
  }

  #[cfg(test)]
  async fn set_prompt(&self, recipient_id: UserId, prompt: Prompt) {
    let (discriminant, payload) = prompt.store();
    let message_id = MessageId(0).store();
    let recipient_id = recipient_id.store();

    sqlx::query!(
      "INSERT INTO prompts
        (discriminant, payload, message_id, recipient_discord_id)
      VALUES
        ($1, $2, $3, $4)
      ON CONFLICT (recipient_discord_id) DO UPDATE SET
        discriminant = $1,
        payload = $2,
        message_id = $3,
        recipient_discord_id = $4",
      discriminant,
      payload,
      message_id,
      recipient_id
    )
    .execute(&self.pool)
    .await
    .unwrap();
  }

  #[cfg(test)]
  async fn response(&self, user: UserId, candidate: UserId) -> bool {
    let user = user.store();
    let candidate = candidate.store();
    let row = sqlx::query!(
      "SELECT response FROM responses WHERE discord_id = $1 AND candidate_id = $2",
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
    db:      Db,
    db_name: String,
  }

  impl TestContext {
    async fn new() -> Self {
      static TEST_DATABASE_NUMBER: AtomicUsize = AtomicUsize::new(0);

      let test_database_number = TEST_DATABASE_NUMBER.fetch_add(1, Ordering::Relaxed);

      let db_name = format!(
        "quwue-test-{}-{}",
        std::time::SystemTime::now()
          .duration_since(std::time::SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_millis(),
        test_database_number,
      );

      let db = Db::connect(&db_name).await.unwrap();

      TestContext { db, db_name }
    }
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn on_disk_database_is_persistant() {
    let TestContext { db, db_name } = TestContext::new().await;

    assert_eq!(db.user_count().await.unwrap(), 0);

    db.create_user(Prompt::Quiescent).await;

    assert_eq!(db.user_count().await.unwrap(), 1);

    drop(db);

    let db = Db::connect(&db_name).await.unwrap();

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
      discord_id,
    };
    assert_eq!(have, want);

    let prompt_message = PromptMessage {
      prompt: Prompt::Welcome,
      message_id,
    };

    let update = Update {
      action:      Some(Action::Welcome),
      next_prompt: Prompt::Welcome,
    };

    let tx = context.db.prepare(have.discord_id, &update).await.unwrap();

    tx.commit(message_id).await.unwrap();

    let have = context.db.user(discord_id).await.unwrap();
    let want = User {
      id: 1,
      welcomed: true,
      prompt_message: Some(prompt_message),
      bio: None,
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
      discord_id,
    };
    assert_eq!(have, want);

    let prompt_message = PromptMessage {
      prompt: Prompt::Bio,
      message_id,
    };

    let update = Update {
      action:      Some(Action::SetBio {
        text: "bio!".to_owned(),
      }),
      next_prompt: Prompt::Bio,
    };

    let tx = context.db.prepare(have.discord_id, &update).await.unwrap();

    tx.commit(message_id).await.unwrap();

    let have = context.db.user(discord_id).await.unwrap();
    let want = User {
      id: 1,
      welcomed: false,
      prompt_message: Some(prompt_message),
      bio: Some("bio!".to_owned()),
      discord_id,
    };
    assert_eq!(have, want);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn expect_candidate() {
    let context = TestContext::new().await;

    let a = context.db.create_user(Prompt::Quiescent).await;
    context.db.create_user(Prompt::Candidate { id: a }).await;
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn filter_out_accepted_candidates() {
    let context = TestContext::new().await;

    let a = context.db.create_user(Prompt::Quiescent).await;
    let b = context.db.create_user(Prompt::Candidate { id: a }).await;

    let update = Update {
      action:      Some(Action::AcceptCandidate { id: a }),
      next_prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(b, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn filter_out_declined_candidates() {
    let context = TestContext::new().await;

    let a = context.db.create_user(Prompt::Quiescent).await;
    let b = context.db.create_user(Prompt::Candidate { id: a }).await;

    let update = Update {
      action:      Some(Action::DeclineCandidate { id: a }),
      next_prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(b, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn filter_out_candidates_that_have_declined_user() {
    let context = TestContext::new().await;

    let a = context.db.create_user(Prompt::Quiescent).await;
    let b = context.db.create_user(Prompt::Candidate { id: a }).await;

    let update = Update {
      action:      Some(Action::DeclineCandidate { id: a }),
      next_prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(b, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);

    tx.commit(MessageId(201)).await.unwrap();

    let update = Update {
      action:      None,
      next_prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn dont_filter_candidates_that_have_accepted_user() {
    let context = TestContext::new().await;

    let a = context.db.create_user(Prompt::Quiescent).await;
    let b = context.db.create_user(Prompt::Candidate { id: a }).await;

    let update = Update {
      action:      Some(Action::AcceptCandidate { id: a }),
      next_prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(b, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);

    tx.commit(MessageId(201)).await.unwrap();

    let update = Update {
      action:      None,
      next_prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Candidate { id: b });
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn allow_multiple_responses() {
    let context = TestContext::new().await;

    let a = context.db.create_user(Prompt::Quiescent).await;
    let b = context.db.create_user(Prompt::Candidate { id: a }).await;

    let update = Update {
      action:      Some(Action::AcceptCandidate { id: a }),
      next_prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(b, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);

    tx.commit(MessageId(201)).await.unwrap();

    assert!(context.db.response(b, a).await);

    let update = Update {
      action:      Some(Action::DeclineCandidate { id: a }),
      next_prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(b, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);

    tx.commit(MessageId(201)).await.unwrap();

    assert!(!context.db.response(b, a).await);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn show_match_prompt_after_mutual_acceptance() {
    let context = TestContext::new().await;

    let a = context.db.create_user(Prompt::Quiescent).await;
    let b = context.db.create_user(Prompt::Candidate { id: a }).await;

    let update = Update {
      action:      Some(Action::AcceptCandidate { id: a }),
      next_prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(b, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Quiescent);

    tx.commit(MessageId(201)).await.unwrap();

    let update = Update {
      action:      Some(Action::AcceptCandidate { id: b }),
      next_prompt: Prompt::Quiescent,
    };

    let tx = context.db.prepare(a, &update).await.unwrap();

    assert_eq!(tx.prompt, Prompt::Match { id: b });

    tx.commit(MessageId(201)).await.unwrap();
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn inserting_responses_from_non_existant_users_is_an_error() {
    let context = TestContext::new().await;

    context.db.create_user(Prompt::Quiescent).await;

    let mut tx = context.db.pool.begin().await.unwrap();

    let error = sqlx::query!(
      "INSERT INTO responses
        (discord_id, candidate_id, response, dismissed)
      VALUES
        (1, 100, TRUE, FALSE)",
    )
    .execute(&mut tx)
    .await
    .unwrap_err();

    guard_unwrap!(let sqlx::Error::Database(error) = error);

    assert_eq!(
      error.message(),
      r#"insert or update on table "responses" violates foreign key constraint "responses_discord_id_fkey""#
    );
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn inserting_responses_to_non_existant_users_is_an_error() {
    let context = TestContext::new().await;

    context.db.create_user(Prompt::Quiescent).await;

    let mut tx = context.db.pool.begin().await.unwrap();

    let error = sqlx::query!(
      "INSERT INTO responses
        (discord_id, candidate_id, response, dismissed)
      VALUES
        (100, 1, TRUE, FALSE)",
    )
    .execute(&mut tx)
    .await
    .unwrap_err();

    guard_unwrap!(let sqlx::Error::Database(error) = error);

    assert_eq!(
      error.message(),
      r#"insert or update on table "responses" violates foreign key constraint "responses_discord_id_fkey""#
    );
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn candidate_function_prioritizes_users_who_have_accepted() {
    let context = TestContext::new().await;

    let a = context.db.create_user(Prompt::Quiescent).await;
    let b = context.db.create_user(Prompt::Candidate { id: a }).await;
    let c = context.db.create_user(Prompt::Candidate { id: a }).await;
    context.db.set_prompt(b, Prompt::Quiescent).await;
    context.db.set_prompt(c, Prompt::Quiescent).await;

    let mut tx = context.db.pool.begin().await.unwrap();
    assert_eq!(Db::get_candidate(&mut tx, a).await.unwrap(), Some(b));

    let update = Update {
      action:      Some(Action::AcceptCandidate { id: a }),
      next_prompt: Prompt::Quiescent,
    };
    context
      .db
      .prepare(c, &update)
      .await
      .unwrap()
      .commit(MessageId(0))
      .await
      .unwrap();
    context.db.set_prompt(c, Prompt::Quiescent).await;

    let mut tx = context.db.pool.begin().await.unwrap();
    assert_eq!(Db::get_candidate(&mut tx, a).await.unwrap(), Some(c));
  }
}
