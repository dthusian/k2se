use std::fmt::Display;

pub trait ResultExt {
  type Output;
  fn pretty_unwrap(self) -> Self::Output;
}

impl<T, E: Display> ResultExt for Result<T, E> {
  type Output = T;

  fn pretty_unwrap(self) -> T {
    self.map_err(|v| panic!("{}", v)).unwrap()
  }
}

