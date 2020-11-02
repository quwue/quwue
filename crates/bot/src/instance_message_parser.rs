use crate::common::*;

#[derive(Clone, Debug)]
pub(crate) struct InstanceMessageParser {
  instance: Instance,
  inner:    RunMessageParser,
}

impl InstanceMessageParser {
  #[cfg(test)]
  pub(crate) fn new(inner: RunMessageParser, instance: Instance) -> Self {
    Self { inner, instance }
  }

  pub(crate) fn parse<'msg>(&self, msg: &'msg str) -> Option<&'msg str> {
    let (instance, msg) = self.inner.parse(msg)?;

    if instance != self.instance {
      return None;
    }

    Some(msg)
  }

  pub(crate) fn prefix_message(&self, msg: &str) -> String {
    self.inner.prefix_message(&self.instance, msg)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! case {
    {
      $run:expr,
      $test_path:expr,
      $user:expr,
      $msg:expr,
      $want:expr $(,)?
    } => {
      let test_path = TestPath::from_test_path_string($test_path);
      let run = RunMessageParser::new($run);
      let instance = Instance::new(test_path, $user);
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
