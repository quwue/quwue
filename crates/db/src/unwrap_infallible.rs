use crate::common::*;

pub(crate) trait UnwrapInfallible {
  type Unwrapped;

  fn unwrap_infallible(self) -> Self::Unwrapped;
}

impl<T> UnwrapInfallible for Result<T, Infallible> {
  type Unwrapped = T;

  fn unwrap_infallible(self) -> Self::Unwrapped {
    self.unwrap()
  }
}
