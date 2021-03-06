#[macro_export]
macro_rules! test_name {
  () => {{
    fn f() {}
    crate::test_name::from_closure_type(f)
  }};
}

pub(crate) fn from_closure_type<T>(f: T) -> String {
  fn type_name_of<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
  }
  let name = type_name_of(f);
  let parts = name.split("::").collect::<Vec<&str>>();
  assert_eq!(parts[parts.len() - 1], "f");
  assert_eq!(parts[parts.len() - 2], "{{closure}}");
  dbg!(name);
  parts[parts.len() - 3].to_owned()
}
