use crate::common::*;

use sqlx::SqlitePool;

#[derive(Debug)]
pub struct Db {
  pool: SqlitePool,
}

macro_rules! load_user {
  {$user:expr} => {
    {
      let user = $user;

      let prompt = user.prompt.map(Prompt::load).transpose()?;
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

      User {
        id:             u64::load(user.id).unwrap_infallible(),
        discord_id:     UserId::load(user.discord_id).unwrap_infallible(),
        welcomed:       user.welcomed,
        bio:            user.bio,
        prompt_message,
      }
    }
  }
}

impl Db {
  pub async fn new() -> Result<Self> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

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
        discord_id != ? LIMIT 1",
      discord_id
    )
    .fetch_optional(tx)
    .await?;

    Ok(row.map(|row| UserId::load(row.discord_id).unwrap_infallible()))
  }

  pub async fn prepare<'a>(&'a self, user: User, update: Update) -> Result<UpdateTx<'a>> {
    let mut tx = self.pool.begin().await?;

    let user_id = user.discord_id;

    if let Some(action) = update.action {
      use Action::*;
      match action {
        Welcome => Self::welcome(&mut tx, user_id).await?,
        SetBio { text } => Self::set_bio(&mut tx, user_id, &text).await?,
      }
    }

    let mut prompt = update.prompt;

    if prompt.quiescent() {
      if let Some(_candidate) = Self::candidate(&mut tx, user.discord_id).await? {
        prompt = Prompt::Candidate;
      }
    };

    let update_tx = UpdateTx {
      user_id,
      prompt,
      tx,
    };

    Ok(update_tx)
  }

  pub(crate) async fn commit<'a>(
    mut tx: Transaction<'a>,
    discord_id: UserId,
    prompt_message: PromptMessage,
  ) -> Result<()> {
    let discord_id = discord_id.store();
    let prompt = Some(prompt_message.prompt.store());
    let prompt_message_id = Some(prompt_message.message_id.store());

    sqlx::query!(
      "UPDATE
        users
      SET
        prompt = ?,
        prompt_message_id = ?
      WHERE discord_id = ?",
      prompt,
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

  #[cfg(test)]
  async fn user_count(&self) -> Result<u64> {
    Ok(
      sqlx::query!("SELECT COUNT(*) as count FROM users")
        .fetch_one(&self.pool)
        .await?
        .count as u64,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test(flavor = "multi_thread")]
  async fn create_user() {
    let db = Db::new().await.unwrap();

    let discord_id = UserId(100);

    assert_eq!(db.user_count().await.unwrap(), 0);

    let have = db.user(discord_id).await.unwrap();
    let want = User {
      id: 1,
      prompt_message: None,
      welcomed: false,
      bio: None,
      discord_id,
    };
    assert_eq!(have, want);

    assert_eq!(db.user_count().await.unwrap(), 1);

    let have = db.user(discord_id).await.unwrap();
    assert_eq!(have, want);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn welcome() {
    let db = Db::new().await.unwrap();

    let discord_id = UserId(100);
    let message_id = MessageId(200);

    let have = db.user(discord_id).await.unwrap();
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
      action: Some(Action::Welcome),
      prompt: Prompt::Welcome,
    };

    let tx = db.prepare(have, update).await.unwrap();

    tx.commit(message_id).await.unwrap();

    let have = db.user(discord_id).await.unwrap();
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
    let db = Db::new().await.unwrap();

    let discord_id = UserId(100);
    let message_id = MessageId(200);

    let have = db.user(discord_id).await.unwrap();
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
      action: Some(Action::SetBio {
        text: "bio!".to_string(),
      }),
      prompt: Prompt::Bio,
    };

    let tx = db.prepare(have, update).await.unwrap();

    tx.commit(message_id).await.unwrap();

    let have = db.user(discord_id).await.unwrap();
    let want = User {
      id: 1,
      welcomed: false,
      prompt_message: Some(prompt_message),
      bio: Some("bio!".to_string()),
      discord_id,
    };
    assert_eq!(have, want);
  }
}
