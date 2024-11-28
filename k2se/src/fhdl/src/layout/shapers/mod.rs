use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use crate::err::Cerr;
use crate::util::invert;

pub mod power_pole_shaper;

pub trait LayoutShaper {
  fn is_free(&self, pos: (i32, i32)) -> bool;
  fn is_free_area(&self, first_corner: (i32, i32), second_corner: (i32, i32)) -> Vec<Vec<bool>>;
}

fn util_prop_error<T, E: Error>(r: Result<T, E>) -> Result<T, Cerr> {
  r.map_err(|v| Cerr::LayoutShaperInvalidArg(format!("{:?}", v)))
}

fn util_get_parse_opt<T: FromStr>(opts: &HashMap<String, String>, k: &str) -> Result<Option<T>, Cerr>
  where T::Err: Error
{
  invert(opts.get(k).map(|v| util_prop_error(v.parse::<T>())))
}

fn util_get_opt(opts: &HashMap<String, String>, k: &str) -> Option<String> {
  opts.get(k).cloned()
}