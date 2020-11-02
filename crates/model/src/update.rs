use crate::common::*;

#[derive(Debug)]
pub struct Update {
  pub prompt: Prompt,
  pub action: Option<Action>,
}
