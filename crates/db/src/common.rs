// stdlib
pub(crate) use std::convert::{Infallible, TryFrom};

// dependencies
pub(crate) use num_enum::TryFromPrimitiveError;
pub(crate) use snafu::{ResultExt, Snafu};
pub(crate) use twilight_model::id::{MessageId, UserId};

// local dependencies
pub(crate) use model::{Action, Prompt, PromptMessage, Update, User};

// modules
pub(crate) use crate::error;

// traits
pub(crate) use crate::{unwrap_infallible::UnwrapInfallible, value::Value};

// structs and enums
pub(crate) use crate::{db::Db, error::Error, update_tx::UpdateTx};

// type aliases
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;
pub(crate) type Transaction<'a> = sqlx::Transaction<'a, sqlx::Sqlite>;
