use crate::common::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
  #[snafu(context(false), display("Database error: {}", source))]
  Sqlx {
    source: sqlx::Error,
  },
  PromptLoad {
    source: TryFromPrimitiveError<Prompt>,
  },
  PromptMessageLoad {
    prompt:     Option<Prompt>,
    message_id: Option<MessageId>,
  },
  Bool {
    storage: i64,
  },
}

impl From<sqlx::migrate::MigrateError> for Error {
  fn from(source: sqlx::migrate::MigrateError) -> Self {
    Self::Sqlx {
      source: source.into(),
    }
  }
}
