use std::backtrace::Backtrace;
use std::fmt::Display;

pub mod imp_iter;

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

pub fn print_backtrace() {
  let cap = Backtrace::force_capture();
  eprintln!("{:?}", cap);
}

pub fn invert<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
  x.map_or(Ok(None), |v| v.map(Some))
}