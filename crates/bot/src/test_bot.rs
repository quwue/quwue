use crate::common::*;

#[macro_export]
macro_rules! test_bot {
  () => {{
    crate::test_bot::TestBot::new(crate::test_name!())
  }};
}

use futures::future::{Map, Shared};
use tokio::{
  sync::oneshot::{error::RecvError, Receiver},
  task::JoinError,
};

type MapResult = fn(Result<JoinError, RecvError>) -> Arc<Result<JoinError, RecvError>>;

pub(crate) type ErrorReceiver = Shared<Map<Receiver<JoinError>, MapResult>>;

pub(crate) struct TestBot {
  error:            ErrorReceiver,
  test_name:        String,
  next_user_number: u64,
  #[allow(unused)]
  bot:              Bot,
}

impl TestBot {
  pub(crate) async fn new(test_name: String) -> Self {
    let test_run = TestDispatcher::get_instance().await.test_run_id();

    let test_id = TestId::new(test_run, test_name.clone());

    let bot = Bot::new_test_instance(test_id)
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

    Self {
      error: rx.map(Arc::new as MapResult).shared(),
      next_user_number: 0,
      test_name,
      bot,
    }
  }

  pub(crate) async fn new_user(&mut self) -> TestUser {
    let next_user_number = self.next_user_number;
    self.next_user_number += 1;
    let test_user_id = TestUserId::new(self.test_name.clone(), next_user_number);
    TestUser::new(self.error.clone(), test_user_id).await
  }
}
