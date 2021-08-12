// stdlib
pub(crate) use std::{
  env,
  fmt::{self, Display, Formatter},
  io,
  ops::Deref,
  panic,
  path::{Path, PathBuf},
  process,
  sync::Arc,
  time::{Duration, Instant},
};

// dependencies
pub(crate) use ::{
  futures_util::StreamExt,
  snafu::{ResultExt, Snafu},
  structopt::StructOpt,
  tokio::{runtime::Runtime, sync::Mutex},
  tracing_log::LogTracer,
  tracing_subscriber::{layer::SubscriberExt, EnvFilter},
  twilight_cache_inmemory::InMemoryCache,
  twilight_gateway::{
    cluster::{ClusterStartError, Events},
    Cluster, EventTypeFlags, Intents,
  },
  twilight_http::{
    api_error::ApiError, client::Client, request::channel::reaction::RequestReactionType,
    Error as HttpError,
  },
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
  model::{Action, Response, User},
};

// logging macros
#[allow(unused)]
pub(crate) use tracing::{error, info, span, trace, warn};

// modules
pub(crate) use crate::{async_static, error, logging, rate_limit, runtime};

// structs and enums
pub(crate) use crate::{
  arguments::Arguments, bot::Bot, error::Error, test_id::TestId, test_message::TestMessage,
  test_run_id::TestRunId, test_user_id::TestUserId,
};

// type aliases
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

// test imports
#[cfg(test)]
mod test {
  // stdlib
  pub(crate) use std::collections::BTreeMap;

  // dependencies
  pub(crate) use ::{
    futures::{
      future::{Future, FutureExt},
      select,
    },
    http::StatusCode,
    once_cell::sync::Lazy,
    serde::Deserialize,
    tempfile::TempDir,
    tokio::{
      sync::{mpsc, RwLock},
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
    test_dispatcher::TestDispatcher, test_event::TestEvent, test_user::TestUser,
  };
}

#[cfg(test)]
pub(crate) use test::*;
