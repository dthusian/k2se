use std::cell::{Cell};
use std::fmt::Display;
use crate::err::{Cerr, CerrSpan};
use crate::parse::span::{Span, WithSpan};
use crate::parse::tokenizer::Token;


/// The main struct used by the AST parser to manipulate `Token`s.
/// Holds an immutable list of tokens that can be accessed with `Cursor`s.
#[derive(Debug, Clone)]
pub struct TokenStream {
  tokens: Vec<WithSpan<Token>>
}

impl TokenStream {
  pub fn from_tokens(tokens: Vec<WithSpan<Token>>) -> Self {
    TokenStream {
      tokens,
    }
  }
  
  pub fn begin(&self) -> Cursor {
    Cursor::new(self)
  }
}

#[derive(Debug, Clone)]
pub struct Cursor<'a> {
  parent: &'a TokenStream,
  position: Cell<usize>,
}

impl<'a> Cursor<'a> {
  /// Constructs a new `Cursor` from a `TokenStream`.
  pub fn new(token_stream: &'a TokenStream) -> Self {
    Cursor {
      parent: token_stream,
      position: Cell::new(0),
    }
  }
  
  /// Gets a token from the stream, returning [`None`] if the stream ends.
  pub fn next_or_eof(&self) -> Option<(&Token, Span)> {
    let pos = self.position.get();
    let t = self.parent.tokens.get(pos)?;
    self.position.replace(pos + 1);
    Some((&t.t, t.span))
  }
  
  /// Gets a token from the stream. Can error if the stream ends.
  pub fn next(&self) -> Result<(&Token, Span), CerrSpan> {
    self.next_or_eof()
      .ok_or(Cerr::UnexpectedEOF.into())
  }
  
  /// Gets a token from the stream and asserts that it is equal to the provided token.
  pub fn next_assert(&self, expected: &Token) -> Result<Span, CerrSpan> {
    let (token, span) = self.next()?;
    if token != expected {
      Err(Cerr::UnexpectedToken(vec![expected.to_string()]).with(span))
    } else {
      Ok(span)
    }
  }
  
  /// Gets a token and applies a fallible function on it.
  pub fn next_map<U>(&self, f: impl FnOnce(&Token) -> Result<U, Cerr>) -> Result<(U, Span), CerrSpan> {
    let (token, span) = self.next()?;
    let t = f(token)
      .map_err(|v| v.with(span))?;
    Ok((t, span))
  }
  
  /// Takes tokens from the stream while the provided predicate is true.
  pub fn take_while(&self, mut pred: impl FnMut(&Token, Span) -> bool) -> Result<Vec<(&Token, Span)>, CerrSpan> {
    todo!()
  }
}