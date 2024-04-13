use std::fmt::Display;
use nom::error::{convert_error, VerboseError};
use nom::{Finish, IResult};

mod parse;

fn nom_test_parse<'a, 'b: 'a, T>(parser: impl Fn(&'a str) -> IResult<&'a str, T, VerboseError<&'a str>>, data: &'b str) -> (&'a str, T) {
  parser(data).finish().map_err(|v| eprintln!("{}", convert_error(data, v))).unwrap()
}

fn pretty_unwrap<T, E: Display>(r: Result<T, E>) -> T {
  r.map_err(|v| eprintln!("{}", v)).unwrap()
}