use nom::multi::{many1};
use crate::parse::{BinaryOp, ExprPart, identifier, parse_int};
use crate::test::nom_test_parse;

#[test]
fn parse_ident_valid1() {
  let ident = nom_test_parse(identifier, "_mon3tr_ tail");
  assert_eq!(ident, (" tail", "_mon3tr_"));
}

#[test]
fn parse_ident_valid2() {
  let ident = nom_test_parse(identifier, "aminger tail");
  assert_eq!(ident, (" tail", "aminger"));
}

#[test]
fn parse_ident_invalid() {
  let ident = identifier("3r2 tail");
  assert!(ident.is_err())
}

#[test]
fn parse_int_valid1() {
  let v = nom_test_parse(parse_int, "727");
  assert_eq!(v, ("", 727))
}

#[test]
fn parse_int_valid2() {
  let v = nom_test_parse(parse_int, "0xbeef");
  assert_eq!(v, ("", 0xbeef))
}

#[test]
fn parse_int_valid3() {
  let v = nom_test_parse(parse_int, "0b101110\t");
  assert_eq!(v, ("\t", 46))
}

#[test]
fn parse_expr_valid1() {
  let expr = nom_test_parse(many1(ExprPart::parser), "2 + 2 == 5");
  let expected = vec![
    ExprPart::Literal(2),
    ExprPart::Op(BinaryOp::Add),
    ExprPart::Literal(2),
    ExprPart::Op(BinaryOp::Eq),
    ExprPart::Literal(5)
  ];
  assert_eq!(expr.1, expected)
}

#[test]
fn parse_expr_valid2() {
  let expr = nom_test_parse(many1(ExprPart::parser), "z == (abs(re(z)) + i * abs(im(z))) ** 2 + c");
  let expected = vec![
    ExprPart::Identifier("z".into()),
    ExprPart::Op(BinaryOp::Eq),
    ExprPart::LBrace,
    ExprPart::Identifier("abs".into()),
    ExprPart::LBrace,
    ExprPart::Identifier("re".into()),
    ExprPart::LBrace,
    ExprPart::Identifier("z".into()),
    ExprPart::RBrace,
    ExprPart::RBrace,
    ExprPart::Op(BinaryOp::Add),
    ExprPart::Identifier("i".into()),
    ExprPart::Op(BinaryOp::Mul),
    ExprPart::Identifier("abs".into()),
    ExprPart::LBrace,
    ExprPart::Identifier("im".into()),
    ExprPart::LBrace,
    ExprPart::Identifier("z".into()),
    ExprPart::RBrace,
    ExprPart::RBrace,
    ExprPart::RBrace,
    ExprPart::Op(BinaryOp::Pow),
    ExprPart::Literal(2),
    ExprPart::Op(BinaryOp::Add),
    ExprPart::Identifier("c".into())
  ];
  assert_eq!(expr.1, expected)
}