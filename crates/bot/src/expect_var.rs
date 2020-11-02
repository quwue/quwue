use crate::common::*;

pub(crate) fn expect_var(key: &str) -> String {
  match env::var(key) {
    Ok(val) => val,
    Err(err) => panic!("Expected environment variable `{}`: {}", key, err),
  }
}
