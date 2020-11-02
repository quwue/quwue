use crate::common::*;

#[derive(Debug)]
pub(crate) enum Letter {
  Message(String),
  Reaction(Emoji),
}
