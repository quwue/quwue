use crate::common::*;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct TestRunId {
  run: u64,
}

impl TestRunId {
  pub(crate) fn new(run: u64) -> Self {
    Self { run }
  }

  pub(crate) fn filter(self, message: &str) -> Option<TestMessage> {
    let test_message = TestMessage::parse(message)?;

    if test_message.test_run_id() != self {
      return None;
    }

    Some(test_message)
  }

  #[cfg(test)]
  pub(crate) fn prefix_message(self, test_user_id: &TestUserId, msg: &str) -> String {
    use std::fmt::Write;
    let mut prefixed = String::new();
    write!(prefixed, "test-{}-{} {}", self.run, test_user_id, msg).unwrap();
    prefixed
  }
}

impl Display for TestRunId {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.run)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn prefix() {
    let run = TestRunId::new(77);
    let user = TestUserId::new("the_test".into(), 100);
    let message = run.prefix_message(&user, "the message");
    assert_eq!(message, "test-77-the_test.100 the message");
  }
}
