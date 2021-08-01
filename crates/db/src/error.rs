use crate::common::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
  #[snafu(context(false), display("Database error: {}", source))]
  Sqlx {
    source: sqlx::Error,
  },
  Bool {
    storage: i64,
  },
  UrlLoad {
    source: url::ParseError,
    text:   String,
  },
  UserMissingBio {
    id: UserId,
  },
  UserUnknown {
    id: UserId,
  },
  PathUnicodeDecode {
    path: PathBuf,
  },
  PromptLoadBadDiscriminant {
    discriminant: u64,
    source:       TryFromPrimitiveError<PromptDiscriminant>,
  },
  PromptLoadMissingPayload {
    discriminant: PromptDiscriminant,
  },
  PromptLoadSuperfluousPayload {
    discriminant: PromptDiscriminant,
    payload:      i64,
  },
  Internal {
    message: String,
  },
}

impl From<sqlx::migrate::MigrateError> for Error {
  fn from(source: sqlx::migrate::MigrateError) -> Self {
    Self::Sqlx {
      source: source.into(),
    }
  }
}
