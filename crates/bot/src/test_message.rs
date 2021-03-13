use crate::common::*;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct TestMessage {
  test_run:             u64,
  pub(crate) test_name: String,
  pub(crate) test_user: u64,
  pub(crate) text:      String,
}

impl TestMessage {
  pub(crate) fn parse(msg: &str) -> Option<Self> {
    let mut chars = msg.chars();

    // skip prefix
    for c in "test-".chars() {
      if chars.next()? != c {
        return None;
      }
    }

    // extract test run
    let mut test_run = None;

    loop {
      let c = chars.next()?;

      if c == '-' {
        break;
      }

      test_run = Some(test_run.unwrap_or(0) * 10 + u64::from(c.to_digit(10)?));
    }

    let test_run = test_run?;

    let rest = chars.as_str();

    // extract test name
    loop {
      if chars.next()? == '.' {
        break;
      }
    }

    let test_name = rest[..rest.len() - chars.as_str().len() - 1].to_owned();

    if test_name.is_empty() {
      return None;
    }

    // parse test user number
    let mut test_user = None;

    loop {
      let c = match chars.next() {
        None | Some(' ') => break,
        Some(c) => c,
      };

      test_user = Some(test_user.unwrap_or(0) * 10 + u64::from(c.to_digit(10)?));
    }

    let test_user = test_user?;

    let text = chars.as_str().to_owned();

    Some(TestMessage {
      test_run,
      test_name,
      test_user,
      text,
    })
  }

  pub(crate) fn test_run_id(&self) -> TestRunId {
    TestRunId::new(self.test_run)
  }

  pub(crate) fn test_user_id(&self) -> TestUserId {
    TestUserId::new(self.test_name.clone(), self.test_user)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    fn assert_some(input: &str, test_run: u64, test_name: &str, test_user: u64, text: &str) {
      let have =
        TestMessage::parse(input).expect(&format!("Failed to parse test message: `{}`", input));

      let want = TestMessage {
        text: text.into(),
        test_name: test_name.into(),
        test_run,
        test_user,
      };

      assert_eq!(have, want, "Unexpected parse of test message `{}`", input);
    }

    assert_some("test-0-bar.1 baz", 0, "bar", 1, "baz");
    assert_some("test-0-bar.1 ", 0, "bar", 1, "");
    assert_some("test-0-bar.1", 0, "bar", 1, "");
    assert_some("test-2-a.3 b", 2, "a", 3, "b");
    assert_some("test-1-a.0 b", 1, "a", 0, "b");
    assert_some("test-1-bar.100 baz", 1, "bar", 100, "baz");
    assert_some("test-0-bar.0 baz", 0, "bar", 0, "baz");
    assert_some("test-00-bar.0 baz", 0, "bar", 0, "baz");
    assert_some("test-00001-bar.0 baz", 1, "bar", 0, "baz");

    assert_eq!(TestMessage::parse("abcd-0-bar.0 baz"), None);
    assert_eq!(TestMessage::parse("test-asdf-bar.0 baz"), None);
    assert_eq!(TestMessage::parse("test--bar.0 baz"), None);
    assert_eq!(TestMessage::parse("test-0-.0 baz"), None);
    assert_eq!(TestMessage::parse("test-0-bar baz"), None);
    assert_eq!(TestMessage::parse("test-0-bar. baz"), None);
    assert_eq!(TestMessage::parse("test-0-bar.a baz"), None);
    assert_eq!(TestMessage::parse("test-0-bar.a0 baz"), None);
  }
}
