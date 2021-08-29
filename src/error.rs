use crate::common::*;

use twilight_http::{
  request::channel::message::create_message::CreateMessageError, response::DeserializeBodyError,
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("Received a response from a bot:\n{:?}", response))]
  BotResponse { response: Response },

  #[snafu(context(false), display("Failed to start cluster: {}", source))]
  ClusterStart { source: ClusterStartError },

  #[snafu(display("Did not get ready event after starting cluster."))]
  ClusterReady { event: Option<(u64, Event)> },

  #[snafu(context(false), display("Error creating message: {}", source))]
  CreateMessage { source: CreateMessageError },

  #[snafu(context(false))]
  Db { source: db::Error },

  #[snafu(
    context(false),
    display("Failed to deserialize response body: {}", source)
  )]
  DeserializeBody { source: DeserializeBodyError },

  #[snafu(context(false), display("Failed to build embed: {}", source))]
  EmbedBuild {
    source: twilight_embed_builder::EmbedError,
  },

  #[snafu(display("Failed to parse embed image URL: {}", source))]
  EmbedImageUrlParse {
    source: url::ParseError,
    text:   String,
  },

  #[snafu(context(false), display("Http error: {}", source))]
  Http { source: HttpError },

  #[snafu(
    context(false),
    display("Failed to create image source URL: {}", source)
  )]
  ImageSourceUrl {
    source: twilight_embed_builder::image_source::ImageSourceUrlError,
  },

  #[snafu(context(false), display("Database migration failed: {}", source))]
  Migration { source: sqlx::migrate::MigrateError },

  #[snafu(display("Received a non-private response:\n{:?}", response))]
  PublicResponse { response: Response },

  #[snafu(display("Failed to initialize runtime: {}", source))]
  Runtime { source: io::Error },

  #[snafu(display("Failed to retrieve `QUWUE_TOKEN` from environment: {}", source))]
  Token { source: env::VarError },

  #[snafu(display("Received unexpected event: {:?}", event.kind()))]
  UnexpectedEvent { event: Event },

  #[snafu(display("No current user."))]
  User,

  #[snafu(display("Failed to retrieve Discord user by ID: {}", user_id))]
  UserUnavailable { user_id: UserId },
}

impl Error {
  pub(crate) fn user_facing_message(&self) -> String {
    match self {
      Self::BotResponse { .. } => "Received a response from a bot".into(),
      Self::ClusterReady { .. } => "Did not get ready event after starting cluster".into(),
      Self::ClusterStart { .. } => "Discord gateway error".into(),
      Self::CreateMessage { .. } => "Failed to send message".into(),
      Self::Db { .. } => "Database error".into(),
      Self::DeserializeBody { .. } => "Failed to deserialize response body".into(),
      Self::EmbedBuild { .. } => "Failed to build embed".into(),
      Self::EmbedImageUrlParse { .. } => "Failed to parse embed image URL".into(),
      Self::Http { source } => {
        if let twilight_http::error::ErrorType::Response { status, error, .. } = source.kind() {
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
          "HTTP error".to_owned()
        }
      },
      Self::ImageSourceUrl { .. } => "Failed to create image source URL".into(),
      Self::Migration { .. } => "Database migration error".into(),
      Self::PublicResponse { .. } => "Received a non-private response".into(),
      Self::Runtime { .. } => "Failed to initialize runtime".into(),
      Self::Token { .. } => "Failed to get authentication token from environment".into(),
      Self::UnexpectedEvent { .. } => "Unexpected event".into(),
      Self::User => "Failed to get current user".into(),
      Self::UserUnavailable { .. } => "Failed to retrieve Discord user by ID".into(),
    }
  }
}
