use crate::parse::{BinaryOp, Expr, identifier, parse_int};
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
  let expr = nom_test_parse(Expr::parser, "2 + 2 == 5");
  let expected = Expr::BinaryOps {
    precedence: 6,
    car: Box::new(Expr::BinaryOps {
      precedence: 3,
      car: Box::new(Expr::Literal { val: 2 }),
      cdr: vec![(BinaryOp::Add, Expr::Literal { val: 2 })],
    }),
    cdr: vec![(BinaryOp::Eq, Expr::Literal { val: 5, })],
  };
  assert_eq!(expr.1, expected)
}

#[test]
fn parse_expr_valid2() {
  let expr = nom_test_parse(Expr::parser, "z == (Re(z) + i * Im(z)) ** 2 + c");
  let expected = Expr::BinaryOps {
    precedence: 6,
    car: Box::new(Expr::Identifier { name: "z".into() }),
    cdr: vec![(BinaryOp::Eq, Expr::BinaryOps {
      precedence: 3,
      car: Box::new(Expr::BinaryOps {
        precedence: 1,
        car: Box::new(Expr::Braced {
          inner: Box::new(Expr::BinaryOps {
            precedence: 3,
            car: Box::new(Expr::FnCall { func: "Re".into(), args: vec![Expr::Identifier { name: "z".to_string() }] }),
            cdr: vec![(BinaryOp::Add, Expr::BinaryOps {
              precedence: 2,
              car: Box::new(Expr::Identifier { name: "i".into() }),
              cdr: vec![(BinaryOp::Mul, Expr::FnCall { func: "Im".into(), args: vec![Expr::Identifier { name: "z".to_string() }] })],
            })],
          }),
        }),
        cdr: vec![(BinaryOp::Pow, Expr::Literal { val: 2 })],
      }),
      cdr: vec![(BinaryOp::Add, Expr::Identifier { name: "c".into() })],
    })],
  };
  assert_eq!(expr.1, expected)
}