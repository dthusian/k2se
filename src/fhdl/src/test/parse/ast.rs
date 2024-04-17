use std::fmt::Debug;
use crate::err::{Cerr, CerrSpan};
use crate::parse::ast::{Expr, Stmt, TriggerKind};
use crate::parse::span::{Pos, Span};
use crate::parse::tokenizer::{BinaryOp, tokenize};
use crate::parse::tokenstream::{Cursor, TokenStream};

fn util_test_parser<T: Debug>(s: &str, parser: impl FnOnce(&Cursor) -> Result<T, CerrSpan>) -> T {
  let spl = s.split("\n").collect::<Vec<_>>();
  let tokens = TokenStream::from_tokens(tokenize(s.chars())
    .collect::<Result<Vec<_>, _>>()
    .map_err(|v| {
      eprintln!("{}", v.format_err("<test input>", &spl).unwrap());
    })
    .unwrap());
  let cursor = tokens.begin();
  let parsed = parser(&cursor)
    .map_err(|v| {
      eprintln!("{}", v.format_err("<test input>", &spl).unwrap());
    })
    .unwrap();
  parsed
}

fn util_test_parser_err<T: Debug>(s: &str, parser: impl FnOnce(&Cursor) -> Result<T, CerrSpan>) -> CerrSpan {
  let spl = s.split("\n").collect::<Vec<_>>();
  let tokens = TokenStream::from_tokens(tokenize(s.chars())
    .collect::<Result<Vec<_>, _>>()
    .map_err(|v| {
      eprintln!("{}", v.format_err("<test input>", &spl).unwrap());
    })
    .unwrap());
  let cursor = tokens.begin();
  let parsed = parser(&cursor)
    .unwrap_err();
  parsed
}

#[test]
pub fn expr_parse_valid() {
  let expr = util_test_parser("z == (abs(re(z)) + i * abs(im(z))) ** 2 + c", Expr::parse);
  let expected = Expr::BinaryOps {
    car: Box::new(Expr::Identifier {
      name: "z".into(),
    }),
    cdr: vec![(BinaryOp::Eq, Expr::BinaryOps {
      car: Box::new(Expr::BinaryOps {
        car: Box::new(Expr::BinaryOps {
          car: Box::new(Expr::FnCall {
            func: "abs".into(),
            args: vec![Expr::FnCall {
              func: "re".into(),
              args: vec![Expr::Identifier {
                name: "z".into(),
              }],
            }],
          }),
          cdr: vec![(BinaryOp::Add, Expr::BinaryOps {
            car: Box::new(Expr::Identifier { name: "i".into() }),
            cdr: vec![(BinaryOp::Mul, Expr::FnCall {
              func: "abs".into(),
              args: vec![Expr::FnCall {
                func: "im".into(),
                args: vec![Expr::Identifier {
                  name: "z".into(),
                }],
              }],
            })],
          })],
        }),
        cdr: vec![(BinaryOp::Pow, Expr::Literal { val: 2 })],
      }),
      cdr: vec![(BinaryOp::Add, Expr::Identifier { name: "c".into() })],
    })],
  };
  assert_eq!(expr, expected);
}

#[test]
pub fn expr_parse_valid_fncalls() {
  let expr = util_test_parser("fn() / fn(1,) % fn(2, fn()) - fn(3, fn(), fn(4),)", Expr::parse);
  let expected = Expr::BinaryOps {
    car: Box::new(Expr::BinaryOps {
      car: Box::new(Expr::FnCall {
        func: "fn".into(),
        args: vec![],
      }),
      cdr: vec![
        (BinaryOp::Div, Expr::FnCall {
          func: "fn".into(),
          args: vec![Expr::Literal { val: 1 }],
        }),
        (BinaryOp::Mod, Expr::FnCall {
          func: "fn".into(),
          args: vec![
            Expr::Literal { val: 2 },
            Expr::FnCall { func: "fn".into(), args: vec![] }
          ],
        })
      ],
    }),
    cdr: vec![(BinaryOp::Sub, Expr::FnCall {
      func: "fn".into(),
      args: vec![
        Expr::Literal { val: 3 },
        Expr::FnCall {
          func: "fn".into(),
          args: vec![],
        },
        Expr::FnCall {
          func: "fn".into(),
          args: vec![Expr::Literal { val: 4 }],
        }
      ],
    })],
  };
  assert_eq!(expr, expected);
}

#[test]
pub fn expr_parse_valid_brackets() {
  let expr = util_test_parser("among_us(((3 * ((2) + (((1)) & (((((((((x))))))))) )))) ** 4,)", Expr::parse);
  let expected = Expr::FnCall {
    func: "among_us".into(),
    args: vec![Expr::BinaryOps {
      car: Box::new(Expr::BinaryOps {
        car: Box::new(Expr::Literal { val: 3 }),
        cdr: vec![(BinaryOp::Mul, Expr::BinaryOps {
          car: Box::new(Expr::Literal { val: 2 }),
          cdr: vec![(BinaryOp::Add, Expr::BinaryOps {
            car: Box::new(Expr::Literal { val: 1 }),
            cdr: vec![(BinaryOp::And, Expr::Identifier { name: "x".into() })],
          })],
        })],
      }),
      cdr: vec![(BinaryOp::Pow, Expr::Literal { val: 4 })],
    }],
  };
  assert_eq!(expr, expected);
}

#[test]
pub fn expr_parse_invalid1() {
  util_test_parser_err("a + + 3", Expr::parse);
}

#[test]
pub fn expr_parse_invalid2() {
  util_test_parser_err("(a", Expr::parse);
}

#[test]
pub fn expr_parse_invalid3() {
  util_test_parser_err("fn(a, 3,", Expr::parse);
}

#[test]
pub fn expr_parse_invalid4() {
  util_test_parser_err("* x;", Expr::parse);
}

#[test]
pub fn expr_parse_invalid5() {
  util_test_parser_err(") a", Expr::parse);
}

#[test]
pub fn stmt_parse_mem_decl() {
  let stmt = util_test_parser("mem reg1;", Stmt::parse).0;
  let expected = Stmt::MemDecl {
    name: "reg1".into(),
  };
  assert_eq!(stmt, expected);
}

#[test]
pub fn stmt_parse_mem_decl_invalid() {
  util_test_parser_err("mem reg1 = 4;", Stmt::parse);
}

#[test]
pub fn stmt_parse_mem_set1() {
  let stmt = util_test_parser("set reg2 = 4 + wire1;", Stmt::parse).0;
  let expected = Stmt::MemSet {
    name: "reg2".into(),
    assign_type: BinaryOp::Assign,
    expr: Expr::BinaryOps {
      car: Box::new(Expr::Literal { val: 4 }),
      cdr: vec![(BinaryOp::Add, Expr::Identifier { name: "wire1".into() })],
    },
  };
  assert_eq!(stmt, expected);
}

#[test]
pub fn stmt_parse_mem_set2() {
  let stmt = util_test_parser("set reg3 += delay(wire1,) ^ wire2;", Stmt::parse).0;
  let expected = Stmt::MemSet {
    name: "reg3".into(),
    assign_type: BinaryOp::AddAssign,
    expr: Expr::BinaryOps {
      car: Box::new(Expr::FnCall {
        func: "delay".into(),
        args: vec![Expr::Identifier { name: "wire1".into() }],
      }),
      cdr: vec![(BinaryOp::Xor, Expr::Identifier { name: "wire2".into() })],
    },
  };
  assert_eq!(stmt, expected);
}

#[test]
pub fn stmt_parse_mem_set_invalid() {
  util_test_parser_err("set reg1 += wire1 = 3;", Stmt::parse);
}

#[test]
pub fn stmt_parse_wire_decl1() {
  let stmt = util_test_parser("wire wire4 = 2 + 2;", Stmt::parse).0;
  let expected = Stmt::WireDecl {
    name: "wire4".into(),
    expr: Some(Expr::BinaryOps {
      car: Box::new(Expr::Literal { val: 2 }),
      cdr: vec![(BinaryOp::Add, Expr::Literal { val: 2 })],
    }),
  };
  assert_eq!(stmt, expected);
}

#[test]
pub fn stmt_parse_wire_decl2() {
  let stmt = util_test_parser("wire wire5;", Stmt::parse).0;
  let expected = Stmt::WireDecl {
    name: "wire5".into(),
    expr: None,
  };
  assert_eq!(stmt, expected);
}

#[test]
pub fn stmt_parse_inst1() {
  let stmt = util_test_parser("inst module6();", Stmt::parse).0;
  let expected = Stmt::ModuleInst {
    module: "module6".into(),
    args: vec![],
  };
  assert_eq!(stmt, expected);
}

#[test]
pub fn stmt_parse_inst2() {
  let stmt = util_test_parser("inst module7(99, (wire1), (wire2 & 1) + 4,);", Stmt::parse).0;
  let expected = Stmt::ModuleInst {
    module: "module7".into(),
    args: vec![
      Expr::Literal { val: 99, },
      Expr::Identifier { name: "wire1".into() },
      Expr::BinaryOps {
        car: Box::new(Expr::BinaryOps {
          car: Box::new(Expr::Identifier { name: "wire2".into() }),
          cdr: vec![(BinaryOp::And, Expr::Literal { val: 1 })]
        }),
        cdr: vec![(BinaryOp::Add, Expr::Literal { val: 4 })],
      }
    ],
  };
  assert_eq!(stmt, expected);
}

#[test]
pub fn stmt_parse_trigger1() {
  let stmt = util_test_parser("trigger clk changed { set thing = 4; wire unused; };", Stmt::parse).0;
  let expected = Stmt::Trigger {
    watching: "clk".to_string(),
    trigger_kind: TriggerKind::Changed,
    statements: vec![
      (Stmt::MemSet {
        name: "thing".to_string(),
        assign_type: BinaryOp::Assign,
        expr: Expr::Literal { val: 4 },
      }, Span { start: Pos::new(1, 22), end: Pos::new(1, 35) }),
      (Stmt::WireDecl {
        name: "unused".to_string(),
        expr: None,
      }, Span { start: Pos::new(1, 37), end: Pos::new(1, 48) })
    ],
  };
  assert_eq!(stmt, expected);
}

#[test]
pub fn stmt_parse_trigger_invalid1() {
  util_test_parser_err("trigger wire2 boo { set thing = 4; wire unused; };", Stmt::parse);
}

#[test]
pub fn stmt_parse_trigger_invalid2() {
  util_test_parser_err("trigger wire2 boo { set thing = 4; wire unused };", Stmt::parse);
}

#[test]
pub fn stmt_parse_trigger_invalid3() {
  util_test_parser_err("trigger wire2 boo { set thing = 4 wire unused; };", Stmt::parse);
}

#[test]
pub fn stmt_parse_trigger_invalid4() {
  util_test_parser_err("trigger wire2 boo { set thing = 4; wire unused; }", Stmt::parse);
}