use crate::common::*;

#[derive(Debug)]
pub struct Update {
  pub action:      Option<Action>,
  pub next_prompt: Prompt,
}
