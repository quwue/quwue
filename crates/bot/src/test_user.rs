use crate::common::*;

#[macro_export]
macro_rules! test_user {
  () => {{
    crate::test_user::TestUser::new(TestUserId::next(crate::test_name!()))
  }};
}

use crate::test_bot::ErrorReceiver;

#[derive(Debug)]
pub(crate) struct TestUser {
  test_dispatcher: &'static TestDispatcher,
  id:              TestUserId,
  error:           ErrorReceiver,
  events:          mpsc::UnboundedReceiver<(MessageId, TestEvent)>,
}

impl TestUser {
  pub(crate) async fn new(error: ErrorReceiver, id: TestUserId) -> Self {
    let test_dispatcher = TestDispatcher::get_instance().await;

    info!("Initializing expect instance for `{}`…", id);

    let events = test_dispatcher.register_test_user(&id).await;

    Self {
      error,
      test_dispatcher,
      id,
      events,
    }
  }

  pub(crate) async fn send_message(&self, msg: &str) {
    self.test_dispatcher.send_message(&self.id, msg).await;
  }

  pub(crate) async fn send_reaction(&self, id: MessageId, emoji: Emoji) {
    self.test_dispatcher.send_reaction(id, emoji).await;
  }

  pub(crate) async fn receive(&mut self) -> (MessageId, TestEvent) {
    select! {
      result = self.events.recv().fuse() => {
        result.expect("channel sender dropped")
      },
      result = self.error.clone() => {
        match result.as_ref() {
          Ok(join_error) => panic!("Quwue failed: {}", join_error),
          Err(recv_error) => panic!("Failed to read from quwue channel: {}", recv_error),
        }
      }
    }
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
    let id = self.expect_message(&prompt.text()).await;
    for emoji in prompt.reactions() {
      assert_eq!(self.expect_reaction(emoji).await, id);
    }
    id
  }
}
