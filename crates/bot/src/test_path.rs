use crate::common::*;

#[macro_export]
macro_rules! test_path {
  () => {{
    fn f() {}
    fn type_name_of<T>(_: T) -> &'static str {
      std::any::type_name::<T>()
    }
    let name = type_name_of(f);
    let parts = name.split("::").collect::<Vec<&str>>();
    assert_eq!(parts[parts.len() - 1], "f");
    assert_eq!(parts[parts.len() - 2], "{{closure}}");
    TestPath::from_test_path_string(parts[parts.len() - 3])
  }};
}

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub(crate) struct TestPath {
  path: String,
}

impl TestPath {
  pub(crate) fn from_test_path_string(path: &str) -> Self {
    Self { path: path.into() }
  }
}

impl Display for TestPath {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.path)
  }
}
