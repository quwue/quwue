use crate::common::*;

use strum::{EnumString, IntoStaticStr};

#[derive(Debug, Eq, PartialEq, EnumString, IntoStaticStr, Copy, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum Emoji {
  ThumbsDown,
  ThumbsUp,
}

impl Emoji {
  pub(crate) fn markup(self) -> String {
    format!(":{}:", self.name())
  }

  pub fn name(self) -> &'static str {
    self.into()
  }

  pub fn char(self) -> char {
    use Emoji::*;
    match self {
      ThumbsDown => '👎',
      ThumbsUp => '👍',
    }
  }

  pub fn from_chars(chars: &str) -> Option<Self> {
    use Emoji::*;
    match chars {
      "👍" => Some(ThumbsUp),
      "👎" => Some(ThumbsDown),
      _ => None,
    }
  }
}

impl Into<RequestReactionType> for Emoji {
  fn into(self) -> RequestReactionType {
    RequestReactionType::Unicode {
      name: self.char().into(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::str::FromStr;

  #[test]
  fn from_str() {
    assert_eq!(Emoji::from_str("thumbsup"), Ok(Emoji::ThumbsUp));
  }

  #[test]
  fn name() {
    assert_eq!(Emoji::ThumbsUp.name(), "thumbsup");
  }

  #[test]
  fn unicode() {
    assert_eq!(Emoji::ThumbsUp.char(), '👍');
  }

  #[test]
  fn markup() {
    assert_eq!(Emoji::ThumbsUp.markup(), ":thumbsup:");
  }
}
