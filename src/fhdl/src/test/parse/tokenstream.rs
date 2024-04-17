use std::cell::Cell;
use crate::err::{Cerr};
use crate::parse::span::{Span, WithSpan};
use crate::parse::tokenizer::Token;
use crate::parse::tokenstream::TokenStream;

#[test]
pub fn cursor_next() {
  let ds = Span::default();
  let s = TokenStream::from_tokens(vec![
    WithSpan::new(ds, Token::Literal(7)),
    WithSpan::new(ds, Token::Literal(6)),
    WithSpan::new(ds, Token::Literal(3)),
    WithSpan::new(ds, Token::Literal(1))
  ]);
  let cursor = s.begin();
  assert_eq!(cursor.next(), Ok((&Token::Literal(7), ds)));
  assert_eq!(cursor.next(), Ok((&Token::Literal(6), ds)));
  assert_eq!(cursor.next(), Ok((&Token::Literal(3), ds)));
  assert_eq!(cursor.next(), Ok((&Token::Literal(1), ds)));
  assert_eq!(cursor.next(), Err(Cerr::UnexpectedEOF.into()));
}

#[test]
pub fn cursor_next_or_eof() {
  let ds = Span::default();
  let s = TokenStream::from_tokens(vec![
    WithSpan::new(ds, Token::Literal(99)),
    WithSpan::new(ds, Token::Literal(22)),
  ]);
  let cursor = s.begin();
  assert_eq!(cursor.next_or_eof(), Some((&Token::Literal(99), ds)));
  assert_eq!(cursor.next_or_eof(), Some((&Token::Literal(22), ds)));
  assert_eq!(cursor.next_or_eof(), None);
}

#[test]
pub fn cursor_next_assert() {
  let ds = Span::default();
  let s = TokenStream::from_tokens(vec![
    WithSpan::new(ds, Token::Literal(20)),
    WithSpan::new(ds, Token::Literal(40))
  ]);
  let cursor = s.begin();
  assert_eq!(cursor.next_assert(&Token::Literal(20)), Ok(ds));
  assert_eq!(cursor.next_assert(&Token::Literal(20)), Err(Cerr::UnexpectedToken(vec!["20".into()]).with(ds)))
}

#[test]
pub fn cursor_next_map() {
  let ds = Span::default();
  let s = TokenStream::from_tokens(vec![
    WithSpan::new(ds, Token::Literal(20)),
    WithSpan::new(ds, Token::Literal(40))
  ]);
  let cursor = s.begin();
  let mapper_called = Cell::new(0);
  let mut mapper = |x: &Token| { mapper_called.set(mapper_called.get() + 1); Ok(x.get_literal().unwrap() + 5) };
  assert_eq!(cursor.next_map(&mut mapper), Ok((25, ds)));
  assert_eq!(mapper_called.get(), 1);
  assert_eq!(cursor.next_map(&mut mapper), Ok((45, ds)));
  assert_eq!(mapper_called.get(), 2);
}

#[test]
pub fn cursor_next_identifier() {
  let ds = Span::default();
  let s = TokenStream::from_tokens(vec![
    WithSpan::new(ds, Token::Literal(20)),
    WithSpan::new(ds, Token::Name("ident".into()))
  ]);
  let cursor = s.begin();
  assert_eq!(cursor.next_identifier(), Err(Cerr::UnexpectedTokenType("identifier").with(ds)));
  assert_eq!(cursor.next_identifier(), Ok(("ident".into(), ds)))
}

#[test]
pub fn cursor_peek() {
  let ds = Span::default();
  let s = TokenStream::from_tokens(vec![
    WithSpan::new(ds, Token::Literal(7)),
    WithSpan::new(ds, Token::Literal(6)),
    WithSpan::new(ds, Token::Literal(3)),
    WithSpan::new(ds, Token::Literal(1))
  ]);
  let cursor = s.begin();
  assert_eq!(cursor.peek(), Ok((&Token::Literal(7), ds)));
  assert_eq!(cursor.peek(), Ok((&Token::Literal(7), ds)));
  assert_eq!(cursor.peek(), Ok((&Token::Literal(7), ds)));
  assert_eq!(cursor.next(), Ok((&Token::Literal(7), ds)));
  assert_eq!(cursor.next(), Ok((&Token::Literal(6), ds)));
  assert_eq!(cursor.peek(), Ok((&Token::Literal(3), ds)));
}

#[test]
pub fn cursor_peek_assert() {
  let ds = Span::default();
  let s = TokenStream::from_tokens(vec![
    WithSpan::new(ds, Token::Literal(7)),
    WithSpan::new(ds, Token::Literal(6)),
    WithSpan::new(ds, Token::Literal(3)),
    WithSpan::new(ds, Token::Literal(1))
  ]);
  let cursor = s.begin();
  assert_eq!(cursor.peek_assert(&Token::Literal(7)), Ok(ds));
  assert_eq!(cursor.peek_assert(&Token::Literal(9)), Err(Cerr::UnexpectedToken(vec!["9".into()]).with(ds)));
  assert_eq!(cursor.peek_assert(&Token::Literal(7)), Ok(ds));
  cursor.next().unwrap();
  cursor.next().unwrap();
  assert_eq!(cursor.peek_assert(&Token::Literal(0)), Err(Cerr::UnexpectedToken(vec!["0".into()]).with(ds)));
  assert_eq!(cursor.peek_assert(&Token::Literal(3)), Ok(ds));
}