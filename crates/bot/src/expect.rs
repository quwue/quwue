use crate::common::*;

#[macro_export]
macro_rules! expect {
  () => {{
    crate::expect::Expect::new(Instance::next(crate::test_path!()))
  }};
}

use futures::future::{Map, Shared};
use tokio::{
  sync::oneshot::{error::RecvError, Receiver},
  task::JoinError,
};

type MapResult = fn(Result<JoinError, RecvError>) -> Arc<Result<JoinError, RecvError>>;

#[derive(Debug)]
pub(crate) struct Expect {
  run:      &'static Run,
  instance: Instance,
  received: Vec<String>,
  error:    Shared<Map<Receiver<JoinError>, MapResult>>,
  bot:      Bot,
  mailbox:  mpsc::UnboundedReceiver<(MessageId, Letter)>,
}

impl Expect {
  pub(crate) async fn new(instance: Instance) -> Self {
    let run = Run::get().await;

    info!("Initializing expect instance for `{}`…", instance);

    let (mailbox, test_message_parser) = run.register(&instance).await;

    let bot = Bot::test(test_message_parser)
      .await
      .expect("Failed to construct quwue instance");

    let clone = bot.clone();
    let handle = tokio::spawn(async move {
      clone.run().await.expect("quwue failed");
    });

    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
      if let Err(err) = handle.await {
        if err.is_panic() {
          tx.send(err).unwrap();
        }
      }
    });

    info!("Quwue instance intialized.");

    Self {
      received: Vec::new(),
      error: rx.map(Arc::new as MapResult).shared(),
      bot,
      run,
      instance,
      mailbox,
    }
  }

  pub(crate) async fn send(&self, msg: &str) {
    self.run.send(&self.instance, msg).await;
  }

  pub(crate) async fn react(&self, id: MessageId, emoji: Emoji) {
    self.run.react(id, emoji).await;
  }

  pub(crate) async fn receive(&mut self) -> (MessageId, Letter) {
    select! {
      result = self.mailbox.recv().fuse() => {
        result.expect("mailbox sender dropped")
      },
      result = self.error.clone() => {
        match result.as_ref() {
          Ok(join_error) => panic!("Quwue failed: {}", join_error),
          Err(recv_error) => panic!("Failed to read from quwue channel: {}", recv_error),
        }
      }
    }
  }

  pub(crate) async fn message(&mut self, want: &str) -> MessageId {
    let (id, letter) = self.receive().await;
    match letter {
      Letter::Message(have) => assert_eq!(have, want, "unexpected message"),
      Letter::Reaction(emoji) => panic!(
        "Got reaction {} but expected message `{}`",
        emoji.char(),
        want
      ),
    };
    id
  }

  pub(crate) async fn reaction(&mut self, want: Emoji) -> MessageId {
    let (id, letter) = self.receive().await;
    match letter {
      Letter::Reaction(have) => assert_eq!(have, want, "unexpected message"),
      Letter::Message(content) => panic!(
        "Got message `{}` but expected reaction {}",
        content,
        want.char(),
      ),
    };
    id
  }

  pub(crate) async fn prompt(&mut self, prompt: Prompt) -> MessageId {
    let id = self.message(&prompt.text()).await;
    for emoji in prompt.reactions() {
      assert_eq!(self.reaction(emoji).await, id);
    }
    id
  }
}
