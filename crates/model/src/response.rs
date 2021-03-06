use crate::common::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Response {
  Message(String),
  Image(Url),
  Reaction(Emoji),
  UnrecognizedReaction(String),
  Custom(EmojiId),
}

impl Response {
  pub fn message(content: impl Into<String>) -> Response {
    Self::Message(content.into())
  }

  pub fn image(url: Url) -> Response {
    Self::Image(url)
  }

  pub fn unicode_reaction(chars: String) -> Response {
    if let Some(emoji) = Emoji::from_chars(&chars) {
      Self::Reaction(emoji)
    } else {
      Self::UnrecognizedReaction(chars)
    }
  }

  pub fn custom_reaction(id: EmojiId) -> Response {
    Self::Custom(id)
  }
}
