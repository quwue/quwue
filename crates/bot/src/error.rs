use crate::common::*;

use twilight_http::request::channel::message::create_message::CreateMessageError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("Failed to retrieve `QUWUE_TOKEN` from environment: {}", source))]
  Token { source: env::VarError },
  #[snafu(context(false), display("Failed to start cluster: {}", source))]
  ClusterStart { source: ClusterStartError },
  #[snafu(context(false), display("Http error: {}", source))]
  Http { source: HttpError },
  #[snafu(context(false), display("Error creating message: {}", source))]
  CreateMessage { source: CreateMessageError },
  #[snafu(display("No current user."))]
  User,
  #[snafu(display("Received unexpected event: {:?}", event.kind()))]
  UnexpectedEvent { event: Event },
  #[snafu(display("Received a non-private response:\n{:?}", response))]
  PublicResponse { response: Response },
  #[snafu(display("Received a response from a bot:\n{:?}", response))]
  BotResponse { response: Response },
  #[snafu(context(false))]
  Db { source: db::Error },
  #[snafu(context(false), display("Database migration failed: {}", source))]
  Migration { source: sqlx::migrate::MigrateError },
  #[snafu(display("Failed to initialize runtime: {}", source))]
  Runtime { source: io::Error },
  #[snafu(display("Failed to retrieve Discord user by ID: {}", user_id))]
  UserUnavailable { user_id: UserId },
}

impl Error {
  pub(crate) fn message(&self) -> String {
    match self {
      Self::Token { .. } => "Failed to get authentication token from environment.".into(),
      Self::ClusterStart { .. } => "Discord gateway error.".into(),
      Self::Http { source } =>
        if let HttpError::Response { status, error, .. } = source {
          if let ApiError::Ratelimited(ratelimited) = error {
            format!(
              "Ratelimited{}, status {}, retry after {}: {}",
              if ratelimited.global { " globally" } else { "" },
              status,
              ratelimited.retry_after,
              ratelimited.message,
            )
          } else {
            format!("HTTP error {}", status)
          }
        } else {
          "HTTP error.".to_owned()
        },
      Self::CreateMessage { .. } => "Failed to send message".into(),
      Self::User => "Failed to get current user".into(),
      Self::UnexpectedEvent { .. } => "Unexpected event".into(),
      Self::PublicResponse { .. } => "Received a non-private response.".into(),
      Self::BotResponse { .. } => "Received a response from a bot.".into(),
      Self::Db { .. } => "Database error.".into(),
      Self::Migration { .. } => "Database migration error.".into(),
      Self::Runtime { .. } => "Failed to initialize runtime.".into(),
      Self::UserUnavailable { .. } => "Failed to retrieve Discord user by ID.".into(),
    }
  }
}
