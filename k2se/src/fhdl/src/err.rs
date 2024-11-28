use std::error::Error;
use crate::parse::span::{Pos, Span};
use crate::parse::tokenizer::BinaryOp;
use std::fmt::{Debug, Display, Formatter, Write};
use std::num::ParseIntError;
use thiserror::Error;

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
  #[error("Invalid expression")]
  InvalidExpr,

  // Validation Errors
  #[error("'{0}' not declared")]
  NotDeclared(String),
  #[error("Multiple declarations for {0}")]
  MultipleDeclarations(String),
  #[error("Cannot write to input port")]
  WriteToInput,
  #[error(
    "Cannot write to wire which has already been '='-assigned to or connected to an output port"
  )]
  MultipleExclusiveWrites,
  #[error("Cannot '='-assign memory outside of a trigger block")]
  MemAssignOutsideOfTrigger,
  #[error("Wrong number of arguments to module instatiation (expected {0})")]
  WrongNumberOfModuleArgs(usize),
  #[error("Cannot nest trigger blocks")]
  NestedTriggerBlocks,
  #[error("In argument {0}: cannot connect expression to out or inout port")]
  ExprForOutInoutPort(usize),
  #[error("Type error: {0}")]
  TypeErrorGeneric(TypeError),
  #[error("Type error: In argument {0} of function '{1}': {2}")]
  TypeErrArgMismatch(usize, String, TypeError),
  #[error("In argument {0} of function {1}: Expected string literal")]
  ExpectedString(usize, String),
  #[error("Unexpected string literal")]
  UnexpectedString,
  #[error("Unknown function '{0}'")]
  UnknownFunction(String),
  #[error("Wrong number of function args (expected {0})")]
  WrongNumberOfFunctionArgs(usize),
  #[error("Cannot use op {0} on two mixed nets")]
  InvalidOpOnMixedNets(BinaryOp),

  // Synthesis Errors
  #[error("Main module '{0}' not found")]
  MainNotFound(String),
  
  // Layout Errors (todo)
  #[error("LayoutShaper invalid argument: {0}")]
  LayoutShaperInvalidArg(String)
}

impl Cerr {
  pub fn with(self, span: Span) -> CerrSpan {
    CerrSpan::new(span, self)
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CerrSpan {
  pub span: Option<Span>,
  pub cerr: Cerr,
}

impl CerrSpan {
  pub fn new(span: Span, cerr: Cerr) -> Self {
    CerrSpan {
      span: Some(span),
      cerr,
    }
  }

  pub fn without_span(cerr: Cerr) -> Self {
    CerrSpan { span: None, cerr }
  }

  pub fn format_err(&self, filename: &str, source: &[&str]) -> Result<String, std::fmt::Error> {
    let span = self.span.unwrap_or_else(|| {
      Span::from(Pos::new(
        source.len() as u32,
        (source[source.len() - 1].len() - 1) as u32,
      ))
    });
    let mut s = String::new();
    writeln!(
      &mut s,
      "at {}:{}:{}: {}",
      filename, span.start.line, span.start.col, self.cerr
    )?;
    for i in span.start.line..=span.end.line {
      let i = i as usize;
      let cstart = if i == span.start.line as usize {
        span.start.col as usize
      } else {
        0usize
      };
      let cend = if i == span.end.line as usize {
        span.end.col as usize
      } else {
        source[i].len() - 1
      };
      writeln!(&mut s, "  {}", source[i - 1])?;
      writeln!(
        &mut s,
        "  {}{}",
        " ".repeat(cstart),
        "^".repeat(cend - cstart + 1)
      )?;
    }
    Ok(s)
  }
}

impl Display for CerrSpan {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if let Some(span) = self.span {
      write!(
        f,
        "at {}:{} - {}:{}: {}",
        span.start.line, span.start.col, span.end.line, span.end.col, self.cerr
      )
    } else {
      write!(f, "at unknown: {}", self.cerr)
    }
  }
}

impl From<Cerr> for CerrSpan {
  fn from(value: Cerr) -> Self {
    CerrSpan::without_span(value)
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeError {
  pub src_ty: String,
  pub dst_ty: String,
}

impl Display for TypeError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "found type {:?} but expected type {:?}",
      self.src_ty, self.dst_ty
    )
  }
}
