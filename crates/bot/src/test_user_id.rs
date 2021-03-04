use crate::common::*;

use std::sync::Mutex;

#[macro_export]
macro_rules! instance {
  () => {
    TestUserId::new(test_path!())
  };
}

lazy_static::lazy_static! {
  static ref NEXT_USER_MAP: Mutex<BTreeMap<TestName, u64>> = Mutex::new(BTreeMap::new());
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct TestUserId {
  test_path: TestName,
  user:      u64,
}

impl TestUserId {
  #[cfg(test)]
  pub(crate) fn next(test_path: TestName) -> Self {
    let mut next_user_map = NEXT_USER_MAP.lock().unwrap();

    if !next_user_map.contains_key(&test_path) {
      next_user_map.insert(test_path.clone(), 0);
    }

    let user = next_user_map.get_mut(&test_path).unwrap();

    let instance = Self::new(test_path, *user);

    *user += 1;

    instance
  }

  pub(crate) fn new(test_path: TestName, user: u64) -> Self {
    Self { test_path, user }
  }
}

impl Display for TestUserId {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}.{}", self.test_path, self.user)
  }
}
