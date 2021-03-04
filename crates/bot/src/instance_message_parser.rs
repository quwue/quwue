use crate::common::*;

#[derive(Clone, Debug)]
pub(crate) struct InstanceMessageParser {
  test_user_id: TestUserId,
  inner:        RunMessageParser,
}

impl InstanceMessageParser {
  #[cfg(test)]
  pub(crate) fn new(inner: RunMessageParser, test_user_id: TestUserId) -> Self {
    Self {
      inner,
      test_user_id,
    }
  }

  pub(crate) fn parse<'msg>(&self, msg: &'msg str) -> Option<&'msg str> {
    let (test_user_id, msg) = self.inner.parse(msg)?;

    if test_user_id != self.test_user_id {
      return None;
    }

    Some(msg)
  }

  pub(crate) fn prefix_message(&self, msg: &str) -> String {
    self.inner.prefix_message(&self.test_user_id, msg)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! case {
    {
      $run:expr,
      $name:expr,
      $user:expr,
      $msg:expr,
      $want:expr $(,)?
    } => {
      let test_name = TestName::from_test_name($name);
      let run = RunMessageParser::new($run);
      let instance = TestUserId::new(test_name, $user);
      let have = InstanceMessageParser::new(run, instance).parse($msg);
      assert_eq!(have, $want);
    }
  }

  #[test]
  #[rustfmt::skip]
  fn parse_message() {
    case!(1, "bar", 0, "test-1-bar.0 baz", Some("baz"));
    case!(1, "bar", 0, "test-1-bar.1 baz", None       );
    case!(1, "foo", 0, "test-1-bar.0 baz", None       );
  }
}
