use crate::common::*;

pub struct UpdateTx<'a> {
  pub(crate) prompt:  Prompt,
  pub(crate) tx:      Transaction<'a>,
  pub(crate) user_id: UserId,
}

impl<'a> UpdateTx<'a> {
  pub fn prompt(&self) -> Prompt {
    self.prompt
  }

  pub async fn commit(self, prompt_message_id: MessageId) -> Result<()> {
    let prompt_message = PromptMessage {
      prompt:     self.prompt,
      message_id: prompt_message_id,
    };

    Db::commit(self.tx, self.user_id, prompt_message).await?;

    Ok(())
  }

  pub fn inner_transaction(&mut self) -> &mut Transaction<'a> {
    &mut self.tx
  }
}
