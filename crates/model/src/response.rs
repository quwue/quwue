use crate::common::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Response {
  Message(String),
  Reaction(Emoji),
  UnrecognizedReaction(String),
  Custom(EmojiId),
}

impl Response {
  pub fn message(content: impl Into<String>) -> Response {
    Self::Message(content.into())
  }

  pub fn unicode_reaction(chars: String) -> Response {
    Emoji::from_chars(&chars).map_or_else(|| Self::UnrecognizedReaction(chars), Self::Reaction)
  }

  pub fn custom_reaction(id: EmojiId) -> Response {
    Self::Custom(id)
  }
}
