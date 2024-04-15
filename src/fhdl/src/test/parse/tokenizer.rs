use std::num::{IntErrorKind};
use crate::err::{Cerr, CerrSpan};
use crate::parse::span::{Pos, Span};
use crate::parse::tokenizer::{BinaryOp, tokenize, Token};
use crate::util::ResultExt;

pub fn util_tokenize(s: &str) -> Result<Vec<Token>, CerrSpan> {
  tokenize(s.chars())
    .map(|v| v.map(|v| v.t))
    .collect::<Result<Vec<_>, _>>()
}

#[test]
pub fn tokenize_valid1() {
  let tokens = util_tokenize("z == (abs(re(z)) + i * abs(im(z))) ** 2 + c")
    .pretty_unwrap();
  let expected = vec![
    Token::Name("z".into()),
    Token::Op(BinaryOp::Eq),
    Token::LParen,
    Token::Name("abs".into()),
    Token::LParen,
    Token::Name("re".into()),
    Token::LParen,
    Token::Name("z".into()),
    Token::RParen,
    Token::RParen,
    Token::Op(BinaryOp::Add),
    Token::Name("i".into()),
    Token::Op(BinaryOp::Mul),
    Token::Name("abs".into()),
    Token::LParen,
    Token::Name("im".into()),
    Token::LParen,
    Token::Name("z".into()),
    Token::RParen,
    Token::RParen,
    Token::RParen,
    Token::Op(BinaryOp::Pow),
    Token::Literal(2),
    Token::Op(BinaryOp::Add),
    Token::Name("c".into())
  ];
  assert_eq!(tokens, expected)
}

#[test]
pub fn tokenize_valid2() {
  let tokens = util_tokenize("z// boo\n==( abs//\n( // comment \n re ( z ) )//\n+i*abs ( im(z)) )** 2 +c; // foo\n")
    .pretty_unwrap();
  let expected = vec![
    Token::Name("z".into()),
    Token::Op(BinaryOp::Eq),
    Token::LParen,
    Token::Name("abs".into()),
    Token::LParen,
    Token::Name("re".into()),
    Token::LParen,
    Token::Name("z".into()),
    Token::RParen,
    Token::RParen,
    Token::Op(BinaryOp::Add),
    Token::Name("i".into()),
    Token::Op(BinaryOp::Mul),
    Token::Name("abs".into()),
    Token::LParen,
    Token::Name("im".into()),
    Token::LParen,
    Token::Name("z".into()),
    Token::RParen,
    Token::RParen,
    Token::RParen,
    Token::Op(BinaryOp::Pow),
    Token::Literal(2),
    Token::Op(BinaryOp::Add),
    Token::Name("c".into()),
    Token::Semicolon
  ];
  assert_eq!(tokens, expected)
}

#[test]
pub fn tokenize_valid3() {
  let tokens = util_tokenize("module foo//\n(inout//\nsingle troll, inout mixed troll2)//\n {wire b = 3 - 2 ** x; mem foo;foo += troll;}")
    .pretty_unwrap();
  let expected = vec![
    Token::Name("module".into()),
    Token::Name("foo".into()),
    Token::LParen,
    Token::Name("inout".into()),
    Token::Name("single".into()),
    Token::Name("troll".into()),
    Token::Comma,
    Token::Name("inout".into()),
    Token::Name("mixed".into()),
    Token::Name("troll2".into()),
    Token::RParen,
    Token::LBrace,
    Token::Name("wire".into()),
    Token::Name("b".into()),
    Token::Op(BinaryOp::Assign),
    Token::Literal(3),
    Token::Op(BinaryOp::Sub),
    Token::Literal(2),
    Token::Op(BinaryOp::Pow),
    Token::Name("x".into()),
    Token::Semicolon,
    Token::Name("mem".into()),
    Token::Name("foo".into()),
    Token::Semicolon,
    Token::Name("foo".into()),
    Token::Op(BinaryOp::AddAssign),
    Token::Name("troll".into()),
    Token::Semicolon,
    Token::RBrace
  ];
  assert_eq!(tokens, expected);
}

#[test]
pub fn tokenize_invalid1() {
  let err = util_tokenize("//\n[3]");
  assert_eq!(err, Err(CerrSpan {
    span: Span { start: Pos::new(2, 0), end: Pos::new(2, 0) },
    cerr: Cerr::InvalidChar,
  }));
}

#[test]
pub fn tokenize_invalid2() {
  let err = util_tokenize(" 3 0xgrr");
  let Err(CerrSpan {
    span: Span { start: Pos { line: 1, col: 3 }, end: Pos { line: 1, col: 7 } },
    cerr: Cerr::InvalidInteger(parse_int_err),
  }) = err else { panic!("match failed: {:?}", err) };
  assert_eq!(parse_int_err.kind(), &IntErrorKind::InvalidDigit);
}

#[test]
pub fn tokenize_invalid3() {
  let err = util_tokenize("3 <> 9");
  assert_eq!(err, Err(CerrSpan {
    span: Span { start: Pos::new(1, 2), end: Pos::new(1, 3) },
    cerr: Cerr::InvalidOperator,
  }))
}