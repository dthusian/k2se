use crate::err::{Cerr, CerrSpan};
use crate::parse::span::{Span, WithSpan};
use crate::parse::tokenstream::TokenStream;

#[test]
pub fn cursor_next() {
  let ds = Span::default();
  let s = TokenStream::from_tokens(vec![
    WithSpan::new(ds, 7),
    WithSpan::new(ds, 6),
    WithSpan::new(ds, 3),
    WithSpan::new(ds, 1)
  ]);
  let cursor = s.begin();
  assert_eq!(cursor.next(), Ok((&7, ds)));
  assert_eq!(cursor.next(), Ok((&6, ds)));
  assert_eq!(cursor.next(), Ok((&3, ds)));
  assert_eq!(cursor.next(), Ok((&1, ds)));
  assert_eq!(cursor.next(), Err(Cerr::UnexpectedEOF.into()));
}

#[test]
pub fn cursor_next_or_eof() {
  let ds = Span::default();
  let s = TokenStream::from_tokens(vec![
    WithSpan::new(ds, 99),
    WithSpan::new(ds, 22),
  ]);
  let cursor = s.begin();
  assert_eq!(cursor.next_or_eof(), Some((&99, ds)));
  assert_eq!(cursor.next_or_eof(), Some((&22, ds)));
  assert_eq!(cursor.next_or_eof(), None);
}

#[test]
pub fn cursor_next_assert1() {
  let ds = Span::default();
  let s = TokenStream::from_tokens(vec![
    WithSpan::new(ds, 20),
    WithSpan::new(ds, 40)
  ]);
  let cursor = s.begin();
  assert_eq!(cursor.next_assert(&20), Ok(ds));
  assert_eq!(cursor.next_assert(&20), Err(Cerr::UnexpectedToken(vec!["20".into()]).with(ds)))
}

