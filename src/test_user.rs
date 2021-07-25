use crate::common::*;

use crate::test_bot::ErrorReceiver;

#[cfg(test)]
fn create_test_png() -> Vec<u8> {
  use image::{DynamicImage, ImageBuffer, ImageOutputFormat, RgbImage};
  let mut image: RgbImage = ImageBuffer::new(100, 100);
  image.fill(0xFF);
  let dynamic = DynamicImage::ImageRgb8(image);
  let mut dst = Vec::new();
  dynamic.write_to(&mut dst, ImageOutputFormat::Png).unwrap();
  dst
}

#[derive(Debug)]
pub(crate) struct TestUser {
  bot:             Bot,
  error:           ErrorReceiver,
  events:          mpsc::UnboundedReceiver<(MessageId, TestEvent)>,
  id:              TestUserId,
  test_dispatcher: &'static TestDispatcher,
}

impl TestUser {
  pub(crate) async fn new(bot: Bot, error: ErrorReceiver, id: TestUserId) -> Self {
    let test_dispatcher = TestDispatcher::get_instance().await;

    info!("Initializing expect instance for `{}`…", id);

    let events = test_dispatcher.register_test_user(&id).await;

    Self {
      bot,
      error,
      events,
      id,
      test_dispatcher,
    }
  }

  pub(crate) async fn send_message(&self, msg: &str) {
    self.test_dispatcher.send_message(&self.id, msg).await;
  }

  pub(crate) async fn send_reaction(&self, id: MessageId, emoji: Emoji) {
    self.test_dispatcher.send_reaction(id, emoji).await;
  }

  pub(crate) async fn send_attachment(&self, filename: &str, data: Vec<u8>) {
    self
      .test_dispatcher
      .send_attachment(&self.id, filename, data)
      .await;
  }

  pub(crate) async fn receive_with_timeout(
    &mut self,
    duration: Duration,
  ) -> Option<(MessageId, TestEvent)> {
    #![allow(clippy::mut_mut)]
    select! {
      result = self.events.recv().fuse() => {
        Some(result.expect("channel sender dropped"))
      },
      result = self.error.clone() => {
        match result.as_ref() {
          Ok(join_error) => panic!("Quwue failed: {}", join_error),
          Err(recv_error) => panic!("Failed to read from quwue channel: {}", recv_error),
        }
      },
      _ = time::sleep(duration).fuse() => {
        None
      }
    }
  }

  pub(crate) async fn receive(&mut self) -> (MessageId, TestEvent) {
    self
      .receive_with_timeout(Duration::from_secs(60))
      .await
      .expect("TestUser::receive timed out!")
  }

  pub(crate) async fn expect_message(&mut self, want: &str) -> MessageId {
    let (id, letter) = self.receive().await;
    match letter {
      TestEvent::Message(have) => assert_eq!(have, want, "unexpected message"),
      TestEvent::Reaction(emoji) => panic!(
        "Got reaction {} but expected message `{}`",
        emoji.char(),
        want
      ),
    };
    id
  }

  pub(crate) async fn expect_reaction(&mut self, want: Emoji) -> MessageId {
    let (id, letter) = self.receive().await;
    match letter {
      TestEvent::Reaction(have) => assert_eq!(have, want, "unexpected message"),
      TestEvent::Message(content) => panic!(
        "Got message `{}` but expected reaction {}",
        content,
        want.char(),
      ),
    };
    id
  }

  pub(crate) async fn expect_prompt(&mut self, prompt: Prompt) -> MessageId {
    let id = self
      .expect_message(
        &self
          .bot
          .db()
          .prompt_text_outside_update_transaction(prompt)
          .await,
      )
      .await;

    for emoji in prompt.reactions() {
      assert_eq!(self.expect_reaction(emoji).await, id);
    }

    id
  }

  pub(crate) async fn expect_nothing(&mut self) {
    if let Some(event) = self.receive_with_timeout(Duration::from_secs(10)).await {
      panic!("Received unexpected message: {:?}", event)
    }
  }

  pub(crate) fn id(&self) -> UserId {
    self.id.to_discord_user_id()
  }

  #[cfg(test)]
  pub(crate) async fn setup(&mut self) {
    self.send_message("hi").await;
    let id = self.expect_prompt(Prompt::Welcome).await;
    self.send_reaction(id, Emoji::ThumbsUp).await;
    self.expect_prompt(Prompt::Bio).await;
    self
      .send_message(&format!("{}'s bio!", self.id.number()))
      .await;
    self.expect_prompt(Prompt::ProfileImage).await;
    self.send_attachment("image.png", create_test_png()).await;
  }
}
