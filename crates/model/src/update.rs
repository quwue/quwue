use crate::common::*;

#[derive(Debug)]
pub struct Update {
  // TODO: rename to `next_prompt` or `prompt_after_update`
  pub prompt: Prompt,
  pub action: Option<Action>,
}
