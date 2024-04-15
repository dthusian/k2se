
use std::fmt::{Debug, Display, Formatter};
use std::num::ParseIntError;
use thiserror::Error;
use crate::parse::span::Span;

#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum Cerr {
  #[error("Invalid character")]
  InvalidChar,
  #[error("Invalid integer")]
  InvalidInteger(#[from] ParseIntError),
  #[error("Invalid operator")]
  InvalidOperator,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CerrSpan {
  pub span: Span,
  pub cerr: Cerr
}

impl CerrSpan {
  pub fn new(span: Span, cerr: Cerr) -> Self {
    CerrSpan {
      span,
      cerr,
    }
  }
  
  pub fn format_err(&self, filename: &str, source: &[&str]) -> Result<String, std::fmt::Error> {
    let mut s = String::new();
    writeln!(&mut s, "at {}:{}:{}:", filename, self.span.start.line, self.span.start.col)?;
    for i in self.span.start.line..=self.span.end.line {
      let cstart = if i == self.span.start.line { self.span.start.col as usize } else { 0usize };
      let cend = if i == self.span.end.line { self.span.end.col as usize } else { source[i].len() };
      writeln!(&mut s, "  {}", source[i])?;
      writeln!(&mut s, "  {}{}", " ".repeat(cstart), "^".repeat(cend - cstart))?;
    }
    s
  }
}

impl Display for CerrSpan {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}