use crate::common::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
  Bool {
    storage: i64,
  },
  Internal {
    message: String,
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
  #[snafu(context(false), display("Database error: {}", source))]
  Sqlx {
    source: sqlx::Error,
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
}

impl From<sqlx::migrate::MigrateError> for Error {
  fn from(source: sqlx::migrate::MigrateError) -> Self {
    Self::Sqlx {
      source: source.into(),
    }
  }
}
