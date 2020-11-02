use crate::common::*;

#[derive(Debug, Copy, Clone)]
pub(crate) struct RunMessageParser {
  run: u64,
}

impl RunMessageParser {
  #[cfg(test)]
  pub(crate) fn new(run: u64) -> Self {
    Self { run }
  }

  pub(crate) fn parse<'msg>(&self, msg: &'msg str) -> Option<(Instance, &'msg str)> {
    let mut chars = msg.chars();

    // skip prefix
    for c in "test-".chars() {
      if chars.next()? != c {
        return None;
      }
    }

    // parse run
    let mut run = None;

    loop {
      let c = chars.next()?;

      if c == '-' {
        break;
      }

      run = Some(run.unwrap_or(0) * 10 + u64::from(c.to_digit(10)?));
    }

    // ignore messages from different runs
    if run? != self.run {
      return None;
    }

    let rest = chars.as_str();

    // extract test path
    loop {
      if chars.next()? == '.' {
        break;
      }
    }

    let path = &rest[..rest.len() - chars.as_str().len() - 1];

    // parse user
    let mut user = None;

    loop {
      let c = chars.next()?;

      if c == ' ' {
        break;
      }

      user = Some(user.unwrap_or(0) * 10 + u64::from(c.to_digit(10)?));
    }

    let message = chars.as_str();

    let instance = Instance::new(TestPath::from_test_path_string(path), user?);

    Some((instance, message))
  }

  #[cfg(test)]
  pub(crate) fn instance_message_parser(self, instance: Instance) -> InstanceMessageParser {
    InstanceMessageParser::new(self, instance)
  }

  pub(crate) fn prefix_message(self, instance: &Instance, msg: &str) -> String {
    let mut prefixed = String::new();
    write!(prefixed, "test-{}-{} {}", self.run, instance, msg).unwrap();
    prefixed
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! case {
    {
      $run:expr,
      $msg:expr,
      $want:expr $(,)?
    } => {
      let have = RunMessageParser::new($run).parse($msg);
      let want = $want.map(|(path, user, content)|
        (Instance::new(TestPath::from_test_path_string(path), user), content)
      );
      assert_eq!(have, want);
    }
  }

  #[test]
  #[rustfmt::skip]
  fn parse_message() {
    case!(1, "test-1-bar.0 baz",     Some(("bar", 0, "baz"  )));
    case!(1, "test-1-bar.100 baz",   Some(("bar", 100, "baz")));
    case!(1, "test-00001-bar.0 baz", Some(("bar", 0, "baz"  )));
    case!(0, "test-0-bar.0 baz",     Some(("bar", 0, "baz"  )));
    case!(0, "test-00-bar.0 baz",    Some(("bar", 0, "baz"  )));
    case!(0, "test-00-bar.0 baz",    Some(("bar", 0, "baz"  )));
    case!(0, "test-00-bar.abc baz",  None                     );
    case!(0, "test--bar.0 baz",      None                     );
    case!(1, "abcd-1-bar.0 baz",     None                     );
    case!(1, "test-2-bar.0 baz",     None                     );
    case!(2, "test-2-bar.0 baz",     Some(("bar", 0, "baz"  )));
    case!(2, "test-2-barbaz",        None                     );
    case!(2, "test-2-barbaz.0 ",     Some(("barbaz", 0, ""  )));
  }
}
