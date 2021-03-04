use crate::common::*;

#[macro_export]
macro_rules! test_name {
  () => {{
    fn f() {}
    TestName::from_closure_type(f)
  }};
}

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub(crate) struct TestName {
  name: String,
}

impl TestName {
  pub(crate) fn from_closure_type<T>(f: T) -> Self {
    fn type_name_of<T>(_: T) -> &'static str {
      std::any::type_name::<T>()
    }
    let name = type_name_of(f);
    let parts = name.split("::").collect::<Vec<&str>>();
    assert_eq!(parts[parts.len() - 1], "f");
    assert_eq!(parts[parts.len() - 2], "{{closure}}");
    dbg!(name);
    TestName::from_test_name(parts[parts.len() - 3])
  }

  pub(crate) fn from_test_name(name: &str) -> Self {
    Self { name: name.into() }
  }
}

impl Display for TestName {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name)
  }
}
