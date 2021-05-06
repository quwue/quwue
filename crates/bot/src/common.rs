// stdlib
pub(crate) use std::{
  collections::BTreeMap,
  env,
  fmt::{self, Display, Formatter, Write},
  io,
  ops::Deref,
  panic, process,
  sync::Arc,
};

// dependencies
pub(crate) use ::{
  futures_util::StreamExt,
  snafu::{ResultExt, Snafu},
  tokio::runtime::Runtime,
  tracing_log::LogTracer,
  tracing_subscriber::{layer::SubscriberExt, EnvFilter},
  twilight_cache_inmemory::InMemoryCache,
  twilight_gateway::{cluster::ClusterStartError, Cluster, EventTypeFlags, Intents},
  twilight_http::{api_error::ApiError, client::Client, Error as HttpError},
  twilight_model::{
    channel::{Channel, ChannelType, Message, ReactionType},
    gateway::{
      event::Event,
      payload::{MessageCreate, ReactionAdd},
    },
    id::{ChannelId, UserId},
  },
};

// local dependencies
pub(crate) use ::{
  db::Db,
  model::{Response, User},
};

// logging macros
#[allow(unused)]
pub(crate) use tracing::{error, info, span, trace, warn};

// modules
pub(crate) use crate::{async_static, error, logging, rate_limit, runtime};

// structs and enums
pub(crate) use crate::{
  bot::Bot, error::Error, test_id::TestId, test_message::TestMessage, test_user_id::TestUserId,
};

// type aliases
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

// test imports
#[cfg(test)]
mod test {
  // stdlib
  pub(crate) use std::time::{Duration, Instant};

  // dependencies
  pub(crate) use ::{
    futures::{
      future::{Future, FutureExt},
      select,
    },
    http::StatusCode,
    once_cell::sync::Lazy,
    serde::Deserialize,
    tokio::{
      sync::{mpsc, Mutex, RwLock},
      time,
    },
    tracing::instrument,
    twilight_model::{
      channel::{GuildChannel, TextChannel},
      guild::{Guild, Member},
      id::MessageId,
    },
  };

  // local dependencies
  pub(crate) use model::{Emoji, Prompt};

  // macros
  pub(crate) use crate::test_bot;

  // functions
  pub(crate) use crate::expect_var::expect_var;

  // structs and enums
  pub(crate) use crate::{
    test_dispatcher::TestDispatcher, test_event::TestEvent, test_run_id::TestRunId,
    test_user::TestUser,
  };
}

#[cfg(test)]
pub(crate) use test::*;
