use crate::common::*;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct TestId {
  test_run:  TestRunId,
  test_name: String,
}

impl TestId {
  pub(crate) fn new(test_run: TestRunId, test_name: String) -> Self {
    Self {
      test_run,
      test_name,
    }
  }

  pub(crate) fn filter(&self, message: &str) -> Option<TestMessage> {
    let test_message = self.test_run.filter(message)?;

    if test_message.test_name != self.test_name {
      return None;
    }

    Some(test_message)
  }

  pub(crate) fn prefix_message(&self, test_user_number: u64, msg: &str) -> String {
    format!(
      "test-{}-{}.{} {}",
      self.test_run, self.test_name, test_user_number, msg
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn prefix() {
    let id = TestId::new(TestRunId::new(77), "the_test".into());
    let message = id.prefix_message(100, "the message");
    assert_eq!(message, "test-77-the_test.100 the message");
  }
}
