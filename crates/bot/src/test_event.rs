use crate::common::*;

#[derive(Debug)]
pub(crate) enum TestEvent {
  Message(String),
  Reaction(Emoji),
}
