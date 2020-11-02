use crate::common::*;

use twilight_model::id::MessageId;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct PromptMessage {
  pub prompt:     Prompt,
  pub message_id: MessageId,
}
