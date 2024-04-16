
use std::fmt::{Debug, Display, Formatter, Write};
use std::num::ParseIntError;
use thiserror::Error;
use crate::parse::span::{Pos, Span};

#[derive(Error, Debug, Clone, Eq, PartialEq)]
pub enum Cerr {
  // Tokenizer Errors
  #[error("Invalid character")]
  InvalidChar,
  #[error("Invalid integer")]
  InvalidInteger(#[from] ParseIntError),
  #[error("Invalid operator")]
  InvalidOperator,
  
  // AST Parse Errors
  #[error("Unexpected token, expected one of {0:?}")]
  UnexpectedToken(Vec<String>),
  #[error("Unexpected token type, expected an {0}")]
  UnexpectedTokenType(&'static str),
  #[error("Unexpected EOF")]
  UnexpectedEOF,
}

impl Cerr {
  pub fn with(self, span: Span) -> CerrSpan {
    CerrSpan::new(span, self)
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CerrSpan {
  pub span: Option<Span>,
  pub cerr: Cerr
}

impl CerrSpan {
  pub fn new(span: Span, cerr: Cerr) -> Self {
    CerrSpan {
      span: Some(span),
      cerr,
    }
  }
  
  pub fn without_span(cerr: Cerr) -> Self {
    CerrSpan {
      span: None,
      cerr,
    }
  }
  
  pub fn format_err(&self, filename: &str, source: &[&str]) -> Result<String, std::fmt::Error> {
    let mut span = self.span.unwrap_or_else(
      || Span::from(Pos::new((source.len() - 1) as u32, (source[source.len() - 1].len() - 1) as u32))
    );
    let mut s = String::new();
    writeln!(&mut s, "at {}:{}:{}: {}", filename, span.start.line, span.start.col, self.cerr)?;
    for i in span.start.line..=span.end.line {
      let i = i as usize;
      let cstart = if i == span.start.line as usize { span.start.col as usize } else { 0usize };
      let cend = if i == span.end.line as usize { span.end.col as usize } else { source[i].len() - 1 };
      writeln!(&mut s, "  {}", source[i])?;
      writeln!(&mut s, "  {}{}", " ".repeat(cstart), "^".repeat(cend - cstart + 1))?;
    }
    Ok(s)
  }
}

impl Display for CerrSpan {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl From<Cerr> for CerrSpan {
  fn from(value: Cerr) -> Self {
    CerrSpan::without_span(value)
  }
}