// stdlib
pub(crate) use std::{
  convert::{Infallible, TryInto},
  path::PathBuf,
  str::FromStr,
};

// dependencies
pub(crate) use {
  num_enum::TryFromPrimitiveError,
  snafu::{ResultExt, Snafu},
  sqlx::{migrate::MigrateDatabase, PgPool, Postgres},
  twilight_model::id::{MessageId, UserId},
};

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
pub(crate) type Transaction<'a> = sqlx::Transaction<'a, sqlx::Postgres>;

#[cfg(test)]
mod test {
  pub(crate) use {
    guard::guard_unwrap,
    std::sync::atomic::{AtomicUsize, Ordering},
  };
}

#[cfg(test)]
pub(crate) use self::test::*;
