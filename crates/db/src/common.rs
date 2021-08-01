// stdlib
pub(crate) use std::{
  convert::{Infallible, TryInto},
  path::{Path, PathBuf},
};

// dependencies
pub(crate) use num_enum::TryFromPrimitiveError;
pub(crate) use snafu::{ResultExt, Snafu};
pub(crate) use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
pub(crate) use twilight_model::id::{MessageId, UserId};
pub(crate) use url::Url;

// local dependencies
pub(crate) use model::{Action, Emoji, Prompt, PromptDiscriminant, PromptMessage, Update, User};

// modules
pub(crate) use crate::error;

// traits
pub(crate) use crate::{unwrap_infallible::UnwrapInfallible, value::Value};

// structs and enums
pub(crate) use crate::{db::Db, error::Error, update_tx::UpdateTx};

// type aliases
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;
pub(crate) type Transaction<'a> = sqlx::Transaction<'a, sqlx::Sqlite>;

#[cfg(test)]
mod test {
  pub(crate) use guard::guard_unwrap;
  pub(crate) use tempfile::{tempdir, TempDir};
}

#[cfg(test)]
pub(crate) use self::test::*;
