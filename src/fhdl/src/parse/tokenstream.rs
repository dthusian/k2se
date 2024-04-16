use std::cell::{Cell};
use std::fmt::Display;
use crate::err::{Cerr, CerrSpan};
use crate::parse::span::{Span, WithSpan};

pub trait TokenLike: Display + PartialEq {}

impl<T: Display + PartialEq> TokenLike for T {}

/// The main struct used by the AST parser to manipulate `Token`s.
/// Holds an immutable list of tokens that can be accessed with `Cursor`s.
#[derive(Debug, Clone)]
pub struct TokenStream<T: TokenLike> {
  tokens: Vec<WithSpan<T>>
}

impl<T: TokenLike> TokenStream<T> {
  pub fn from_tokens(tokens: Vec<WithSpan<T>>) -> Self {
    TokenStream {
      tokens,
    }
  }
  
  pub fn begin(&self) -> Cursor<T> {
    Cursor::new(self)
  }
}

#[derive(Debug, Clone)]
pub struct Cursor<'a, T: TokenLike> {
  parent: &'a TokenStream<T>,
  position: Cell<usize>,
}

impl<'a, T: TokenLike> Cursor<'a, T> {
  /// Constructs a new `Cursor` from a `TokenStream`.
  pub fn new(token_stream: &'a TokenStream<T>) -> Self {
    Cursor {
      parent: token_stream,
      position: Cell::new(0),
    }
  }
  
  /// Gets a token from the stream, returning [`None`] if the stream ends.
  pub fn next_or_eof(&self) -> Option<(&T, Span)> {
    let pos = self.position.get();
    let t = self.parent.tokens.get(pos)?;
    self.position.replace(pos + 1);
    Some((&t.t, t.span))
  }
  
  /// Gets a token from the stream. Can error if the stream ends.
  pub fn next(&self) -> Result<(&T, Span), CerrSpan> {
    self.next_or_eof()
      .ok_or(Cerr::UnexpectedEOF.into())
  }
  
  /// Gets a token from the stream and asserts that it is equal to the provided token.
  pub fn next_assert(&self, expected: &T) -> Result<Span, CerrSpan> {
    let (token, span) = self.next()?;
    if token != expected {
      Err(Cerr::UnexpectedToken(vec![expected.to_string()]).with(span))
    } else {
      Ok(span)
    }
  }
  
  /// Gets a token and applies a fallible function on it.
  pub fn next_map<U>(&self, f: impl FnOnce(&T) -> Result<U, Cerr>) -> Result<(U, Span), CerrSpan> {
    let (token, span) = self.next()?;
    let t = f(token)
      .map_err(|v| v.with(span))?;
    Ok((t, span))
  }
  
  /// Takes tokens from the stream while the provided predicate is true.
  pub fn take_while(&self, mut pred: impl FnMut(&T, Span) -> bool) -> Result<Vec<(&T, Span)>, CerrSpan> {
    todo!()
  }
}