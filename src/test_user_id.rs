use crate::common::*;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct TestUserId {
  test_name: String,
  user:      u64,
}

impl TestUserId {
  pub(crate) fn new(test_name: String, user: u64) -> Self {
    Self { test_name, user }
  }

  pub(crate) fn to_discord_user_id(&self) -> UserId {
    UserId(self.user)
  }

  pub(crate) fn number(&self) -> u64 {
    self.user
  }
}

impl Display for TestUserId {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}.{}", self.test_name, self.user)
  }
}
