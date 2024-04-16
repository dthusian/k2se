use crate::err::{Cerr, CerrSpan};
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
pub fn cursor_next_assert1() {
  let ds = Span::default();
  let s = TokenStream::from_tokens(vec![
    WithSpan::new(ds, Token::Literal(20)),
    WithSpan::new(ds, Token::Literal(40))
  ]);
  let cursor = s.begin();
  assert_eq!(cursor.next_assert(&Token::Literal(20)), Ok(ds));
  assert_eq!(cursor.next_assert(&Token::Literal(20)), Err(Cerr::UnexpectedToken(vec!["20".into()]).with(ds)))
}

