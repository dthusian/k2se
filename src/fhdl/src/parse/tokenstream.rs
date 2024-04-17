use std::cell::{Cell};
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
  
  /// Takes the next token, asserting that it is [`Token::Name`] and
  /// then unwrapping its inner string.
  pub fn next_identifier(&self) -> Result<(String, Span), CerrSpan> {
    let (id, span) = self
      .next_map(|v| v
        .get_name()
        .map(|v| v.to_owned())
        .ok_or(Cerr::UnexpectedTokenType("identifier")))?;
    Ok((id.to_owned(), span))
  }
  
  /// Peeks the next token without taking it.
  pub fn peek(&self) -> Result<(&Token, Span), CerrSpan> {
    self.peek_or_eof()
      .ok_or(Cerr::UnexpectedEOF.into())
  }
  
  pub fn peek_assert(&self, expected: &Token) -> Result<Span, CerrSpan> {
    let (token, span) = self.peek()?;
    if token != expected {
      Err(Cerr::UnexpectedToken(vec![expected.to_string()]).with(span))
    } else {
      Ok(span)
    }
  }
  
  pub fn peek_or_eof(&self) -> Option<(&Token, Span)> {
    self.parent.tokens.get(self.position.get())
      .map(|v| (&v.t, v.span))
  }
  
  /// Returns true if there were enough elements to skip, false if EOF was
  /// reached first.
  pub fn skip(&self, n: usize) -> bool {
    self.position.set(self.position.get() + n);
    if self.position.get() > self.parent.tokens.len() {
      false
    } else {
      true
    }
  }

  /// Returns true if the rewind went past the starting token.
  pub fn rewind(&self, n: usize) -> bool {
    let end = self.position.get().checked_sub(n);
    if let Some(end) = end {
      self.position.set(end);
      true
    } else {
      self.position.set(0);
      false
    }
  }
  
  /// Takes tokens from the stream while the provided predicate is true.
  /// Does not consume any elements that are not returned.
  pub fn take_while(&self, mut pred: impl FnMut(&Token, Span) -> bool) -> Result<Vec<(&Token, Span)>, CerrSpan> {
    let mut buf = vec![];
    loop {
      let (token, span) = self.next()?;
      if pred(token, span) {
        buf.push((token, span));
      } else {
        self.rewind(1);
        break;
      }
    }
    Ok(buf)
  }
  
  /// Takes and discards tokens until a token passes the specified predicate, returning it
  pub fn search_for(&self, mut pred: impl FnMut(&Token, Span) -> bool) -> Result<(&Token, Span), CerrSpan> {
    loop {
      let (token, span) = self.next()?;
      if pred(token, span) {
        return Ok((token, span));
      }
    }
  }
}