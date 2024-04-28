use crate::err::{Cerr, TypeError};
use crate::parse::ast::{Expr, Module, NetType, PortClass, PortDecl, Stmt, TriggerKind};
use crate::parse::span::Span;
use crate::parse::tokenizer::BinaryOp;
use crate::synth::builtins::{BuiltinFunction, FunctionArgReq, SynthRef};
use crate::synth::synth::{IncompleteNetID, ModuleSynthState};
use crate::synth::transform::transform_modules;
use std::collections::HashMap;
use std::fmt::{Debug};
use std::slice;

/// A builtin function used for testing.
#[derive(Debug)]
struct TestFunction {
  arg_ty: FunctionArgReq,
}

impl BuiltinFunction for TestFunction {
  fn arg_ty(&self) -> &[FunctionArgReq] {
    slice::from_ref(&self.arg_ty)
  }

  fn return_ty(&self) -> NetType {
    NetType::Single
  }

  fn synthesize(&self, _: &mut ModuleSynthState, _: &[SynthRef], _: IncompleteNetID) -> Result<(), Cerr> {
    panic!("not implemented")
  }

  fn constant_fold(&self, _: &[SynthRef]) -> Option<i32> {
    None
  }
}

fn test_builtins() -> HashMap<String, Box<dyn BuiltinFunction>> {
  let mut map = HashMap::<String, Box<dyn BuiltinFunction>>::new();
  map.insert(
    "test1".into(),
    Box::new(TestFunction {
      arg_ty: FunctionArgReq::Net(NetType::Single),
    }),
  );
  map.insert(
    "test2".into(),
    Box::new(TestFunction {
      arg_ty: FunctionArgReq::Net(NetType::Mixed),
    }),
  );
  map.insert(
    "test3".into(),
    Box::new(TestFunction {
      arg_ty: FunctionArgReq::SingleOrLit,
    }),
  );
  map.insert(
    "test4".into(),
    Box::new(TestFunction {
      arg_ty: FunctionArgReq::String,
    }),
  );
  map
}

#[test]
pub fn transform_err_undeclared() {
  let ds = Span::default();
  let ast = vec![(
    Module {
      name: "invalid_module".into(),
      ports: vec![PortDecl {
        port_class: PortClass::Out,
        signal_class: NetType::Single,
        name: "port".into(),
      }],
      stmts: vec![
        (
          Stmt::WireDecl {
            name: "wire1".into(),
            signal_class: NetType::Single,
            expr: Some(Expr::BinaryOps {
              car: Box::new(Expr::Identifier {
                name: "undeclared1".into(),
              }),
              cdr: vec![(BinaryOp::Add, Expr::Literal { val: 2 })],
            }),
          },
          ds,
        ),
        (
          Stmt::Set {
            name: "port".into(),
            assign_type: BinaryOp::Assign,
            expr: Expr::Identifier {
              name: "undeclared2".into(),
            },
          },
          ds,
        ),
        (
          Stmt::Trigger {
            watching: "undeclared3".into(),
            trigger_kind: TriggerKind::Increasing,
            statements: vec![],
          },
          ds,
        ),
        (
          Stmt::ModuleInst {
            module: "undeclared4".into(),
            args: vec![],
          },
          ds,
        ),
      ],
    },
    ds,
  )];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![
    Cerr::NotDeclared("undeclared1".into()).with(ds),
    Cerr::NotDeclared("undeclared2".into()).with(ds),
    Cerr::NotDeclared("undeclared3".into()).with(ds),
    Cerr::NotDeclared("undeclared4".into()).with(ds),
  ];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_multiple_decl() {
  let ds = Span::default();
  let ast = vec![
    (
      Module {
        name: "multi1".into(),
        ports: vec![
          PortDecl {
            port_class: PortClass::In,
            signal_class: NetType::Single,
            name: "multi2".into(),
          },
          PortDecl {
            port_class: PortClass::In,
            signal_class: NetType::Single,
            name: "multi2".into(),
          },
        ],
        stmts: vec![],
      },
      ds,
    ),
    (
      Module {
        name: "multi1".into(),
        ports: vec![],
        stmts: vec![
          (
            Stmt::WireDecl {
              name: "multi4".into(),
              signal_class: NetType::Single,
              expr: None,
            },
            ds,
          ),
          (
            Stmt::MemDecl {
              name: "multi4".into(),
              signal_class: NetType::Single,
            },
            ds,
          ),
          (
            Stmt::WireDecl {
              name: "multi3".into(),
              signal_class: NetType::Single,
              expr: None,
            },
            ds,
          ),
          (
            Stmt::WireDecl {
              name: "multi3".into(),
              signal_class: NetType::Single,
              expr: None,
            },
            ds,
          ),
          (
            Stmt::MemDecl {
              name: "multi5".into(),
              signal_class: NetType::Single,
            },
            ds,
          ),
          (
            Stmt::MemDecl {
              name: "multi5".into(),
              signal_class: NetType::Single,
            },
            ds,
          ),
        ],
      },
      ds,
    ),
  ];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![
    Cerr::MultipleDeclarations("multi1".into()).with(ds),
    Cerr::MultipleDeclarations("multi2".into()).with(ds),
    Cerr::MultipleDeclarations("multi4".into()).with(ds),
    Cerr::MultipleDeclarations("multi3".into()).with(ds),
    Cerr::MultipleDeclarations("multi5".into()).with(ds),
  ];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_writes_to_input() {
  let ds = Span::default();
  let ast = vec![(
    Module {
      name: "invalid_module".into(),
      ports: vec![PortDecl {
        port_class: PortClass::In,
        signal_class: NetType::Single,
        name: "inputter".into(),
      }],
      stmts: vec![(
        Stmt::Set {
          name: "inputter".into(),
          assign_type: BinaryOp::AddAssign,
          expr: Expr::Literal { val: 5 },
        },
        ds,
      )],
    },
    ds,
  )];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![Cerr::WriteToInput.with(ds)];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_multiple_excl_assign1() {
  let ds = Span::default();
  let ast = vec![(
    Module {
      name: "invalid_module".into(),
      ports: vec![],
      stmts: vec![
        (
          Stmt::WireDecl {
            name: "w1".into(),
            signal_class: NetType::Single,
            expr: None,
          },
          ds,
        ),
        (
          Stmt::Set {
            name: "w1".into(),
            assign_type: BinaryOp::Assign,
            expr: Expr::Literal { val: 2 },
          },
          ds,
        ),
        (
          Stmt::Set {
            name: "w1".into(),
            assign_type: BinaryOp::Assign,
            expr: Expr::Literal { val: 2 },
          },
          ds,
        ),
        (
          Stmt::WireDecl {
            name: "w2".into(),
            signal_class: NetType::Single,
            expr: Some(Expr::Literal { val: 4 }),
          },
          ds,
        ),
        (
          Stmt::Set {
            name: "w2".into(),
            assign_type: BinaryOp::AddAssign,
            expr: Expr::Literal { val: 4 },
          },
          ds,
        ),
      ],
    },
    ds,
  )];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![
    Cerr::MultipleExclusiveWrites.with(ds),
    Cerr::MultipleExclusiveWrites.with(ds),
  ];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_multiple_excl_assign2() {
  let ds = Span::default();
  let ast = vec![
    (
      Module {
        name: "excl_writes".into(),
        ports: vec![PortDecl {
          port_class: PortClass::Out,
          signal_class: NetType::Single,
          name: "writee".into(),
        }],
        stmts: vec![],
      },
      ds,
    ),
    (
      Module {
        name: "invalid_module".into(),
        ports: vec![],
        stmts: vec![
          (
            Stmt::WireDecl {
              name: "w1".into(),
              signal_class: NetType::Single,
              expr: None,
            },
            ds,
          ),
          (
            Stmt::ModuleInst {
              module: "excl_writes".into(),
              args: vec![Expr::Identifier { name: "w1".into() }],
            },
            ds,
          ),
          (
            Stmt::ModuleInst {
              module: "excl_writes".into(),
              args: vec![Expr::Identifier { name: "w1".into() }],
            },
            ds,
          ),
        ],
      },
      ds,
    ),
  ];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![Cerr::MultipleExclusiveWrites.with(ds)];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_nested_trigger() {
  let ds = Span::default();
  let ast = vec![(
    Module {
      name: "invalid_module".into(),
      ports: vec![PortDecl {
        port_class: PortClass::In,
        signal_class: NetType::Single,
        name: "w1".into(),
      }],
      stmts: vec![(
        Stmt::Trigger {
          watching: "w1".into(),
          trigger_kind: TriggerKind::Raw,
          statements: vec![(
            Stmt::Trigger {
              watching: "w1".into(),
              trigger_kind: TriggerKind::Raw,
              statements: vec![],
            },
            ds,
          )],
        },
        ds,
      )],
    },
    ds,
  )];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![Cerr::NestedTriggerBlocks.with(ds)];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_bare_mem_assign() {
  let ds = Span::default();
  let ast = vec![(
    Module {
      name: "invalid_module".into(),
      ports: vec![],
      stmts: vec![
        (
          Stmt::MemDecl {
            name: "reg".into(),
            signal_class: NetType::Single,
          },
          ds,
        ),
        (
          Stmt::Set {
            name: "reg".into(),
            assign_type: BinaryOp::Assign,
            expr: Expr::Literal { val: 9 },
          },
          ds,
        ),
      ],
    },
    ds,
  )];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![Cerr::MemAssignOutsideOfTrigger.with(ds)];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_arity_mismatch() {
  let ds = Span::default();
  let ast = vec![
    (
      Module {
        name: "target_module".into(),
        ports: vec![
          PortDecl {
            port_class: PortClass::In,
            signal_class: NetType::Single,
            name: "p1".into(),
          },
          PortDecl {
            port_class: PortClass::Out,
            signal_class: NetType::Single,
            name: "p2".into(),
          },
        ],
        stmts: vec![],
      },
      ds,
    ),
    (
      Module {
        name: "invalid_module".into(),
        ports: vec![],
        stmts: vec![
          (
            Stmt::WireDecl {
              name: "w1".into(),
              signal_class: NetType::Single,
              expr: None,
            },
            ds,
          ),
          (
            Stmt::ModuleInst {
              module: "target_module".into(),
              args: vec![
                Expr::Identifier { name: "w1".into() },
                Expr::Identifier { name: "w1".into() },
                Expr::Identifier { name: "w1".into() },
              ],
            },
            ds,
          ),
        ],
      },
      ds,
    ),
  ];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![Cerr::WrongNumberOfModuleArgs(2).with(ds)];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_expr_for_out_inout() {
  let ds = Span::default();
  let ast = vec![
    (
      Module {
        name: "target_module".into(),
        ports: vec![
          PortDecl {
            port_class: PortClass::InOut,
            signal_class: NetType::Single,
            name: "p1".into(),
          },
          PortDecl {
            port_class: PortClass::Out,
            signal_class: NetType::Single,
            name: "p2".into(),
          },
        ],
        stmts: vec![],
      },
      ds,
    ),
    (
      Module {
        name: "invalid_module".into(),
        ports: vec![],
        stmts: vec![(
          Stmt::ModuleInst {
            module: "target_module".into(),
            args: vec![
              Expr::BinaryOps {
                car: Box::new(Expr::Literal { val: 2 }),
                cdr: vec![(BinaryOp::Add, Expr::Literal { val: 2 })],
              },
              Expr::BinaryOps {
                car: Box::new(Expr::Literal { val: 2 }),
                cdr: vec![(BinaryOp::Add, Expr::Literal { val: 2 })],
              },
            ],
          },
          ds,
        )],
      },
      ds,
    ),
  ];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![
    Cerr::ExprForOutInoutPort(0).with(ds),
    Cerr::ExprForOutInoutPort(1).with(ds),
  ];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_assign_type_error1() {
  let ds = Span::default();
  let ast = vec![(
    Module {
      name: "invalid_module".into(),
      ports: vec![],
      stmts: vec![(
        Stmt::WireDecl {
          name: "w1".into(),
          signal_class: NetType::Mixed,
          expr: Some(Expr::Literal { val: 46 }),
        },
        ds,
      )],
    },
    ds,
  )];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![Cerr::TypeErrorGeneric(TypeError {
    src_ty: "Single".into(),
    dst_ty: "Mixed".into(),
  })
  .with(ds)];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_assign_type_error2() {
  let ds = Span::default();
  let ast = vec![(
    Module {
      name: "invalid_module".into(),
      ports: vec![],
      stmts: vec![
        (
          Stmt::WireDecl {
            name: "w1".into(),
            signal_class: NetType::Mixed,
            expr: None,
          },
          ds,
        ),
        (
          Stmt::WireDecl {
            name: "w2".into(),
            signal_class: NetType::Single,
            expr: Some(Expr::Identifier { name: "w1".into() }),
          },
          ds,
        ),
      ],
    },
    ds,
  )];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![Cerr::TypeErrorGeneric(TypeError {
    src_ty: "Mixed".into(),
    dst_ty: "Single".into(),
  })
  .with(ds)];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_arg_type_error() {
  let ds = Span::default();
  let ast = vec![(
    Module {
      name: "invalid_module".into(),
      ports: vec![],
      stmts: vec![
        (
          Stmt::WireDecl {
            name: "w1".into(),
            signal_class: NetType::Single,
            expr: None,
          },
          ds,
        ),
        (
          Stmt::WireDecl {
            name: "w2".into(),
            signal_class: NetType::Mixed,
            expr: None,
          },
          ds,
        ),
        (
          Stmt::WireDecl {
            name: "dst".into(),
            signal_class: NetType::Single,
            expr: None,
          },
          ds,
        ),
        (
          Stmt::Set {
            name: "dst".into(),
            assign_type: BinaryOp::AddAssign,
            expr: Expr::FnCall {
              func: "test1".into(),
              args: vec![Expr::Identifier { name: "w2".into() }],
            },
          },
          ds,
        ),
        (
          Stmt::Set {
            name: "dst".into(),
            assign_type: BinaryOp::AddAssign,
            expr: Expr::FnCall {
              func: "test1".into(),
              args: vec![Expr::Literal { val: 1 }],
            },
          },
          ds,
        ),
        (
          Stmt::Set {
            name: "dst".into(),
            assign_type: BinaryOp::AddAssign,
            expr: Expr::FnCall {
              func: "test2".into(),
              args: vec![Expr::Identifier { name: "w1".into() }],
            },
          },
          ds,
        ),
        (
          Stmt::Set {
            name: "dst".into(),
            assign_type: BinaryOp::AddAssign,
            expr: Expr::FnCall {
              func: "test2".into(),
              args: vec![Expr::Literal { val: 1 }],
            },
          },
          ds,
        ),
        (
          Stmt::Set {
            name: "dst".into(),
            assign_type: BinaryOp::AddAssign,
            expr: Expr::FnCall {
              func: "test3".into(),
              args: vec![Expr::Identifier { name: "w2".into() }],
            },
          },
          ds,
        ),
      ],
    },
    ds,
  )];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![
    Cerr::TypeErrArgMismatch(
      0,
      "test1".into(),
      TypeError {
        src_ty: "Mixed".into(),
        dst_ty: "Single".into(),
      },
    )
    .with(ds),
    Cerr::TypeErrArgMismatch(
      0,
      "test1".into(),
      TypeError {
        src_ty: "Literal".into(),
        dst_ty: "Single".into(),
      },
    )
    .with(ds),
    Cerr::TypeErrArgMismatch(
      0,
      "test2".into(),
      TypeError {
        src_ty: "Single".into(),
        dst_ty: "Mixed".into(),
      },
    )
    .with(ds),
    Cerr::TypeErrArgMismatch(
      0,
      "test2".into(),
      TypeError {
        src_ty: "Literal".into(),
        dst_ty: "Mixed".into(),
      },
    )
    .with(ds),
    Cerr::TypeErrArgMismatch(
      0,
      "test3".into(),
      TypeError {
        src_ty: "Mixed".into(),
        dst_ty: "Single".into(),
      },
    )
    .with(ds),
  ];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_unknown_func() {
  let ds = Span::default();
  let ast = vec![(
    Module {
      name: "invalid_module".into(),
      ports: vec![],
      stmts: vec![(
        Stmt::WireDecl {
          name: "w1".into(),
          signal_class: NetType::Single,
          expr: Some(Expr::FnCall {
            func: "not_exist".into(),
            args: vec![],
          }),
        },
        ds,
      )],
    },
    ds,
  )];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![Cerr::UnknownFunction("not_exist".into()).with(ds)];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_wrong_number_of_args() {
  let ds = Span::default();
  let ast = vec![(
    Module {
      name: "invalid_module".into(),
      ports: vec![],
      stmts: vec![(
        Stmt::WireDecl {
          name: "w1".into(),
          signal_class: NetType::Single,
          expr: Some(Expr::FnCall {
            func: "test3".into(),
            args: vec![Expr::Literal { val: 4 }, Expr::Literal { val: 4 }],
          }),
        },
        ds,
      )],
    },
    ds,
  )];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![Cerr::WrongNumberOfFunctionArgs(1).with(ds)];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_expected_string() {
  let ds = Span::default();
  let ast = vec![(
    Module {
      name: "invalid_module".into(),
      ports: vec![],
      stmts: vec![
        (
          Stmt::WireDecl {
            name: "w1".into(),
            signal_class: NetType::Single,
            expr: None,
          },
          ds,
        ),
        (
          Stmt::WireDecl {
            name: "w2".into(),
            signal_class: NetType::Single,
            expr: Some(Expr::FnCall {
              func: "test4".into(),
              args: vec![Expr::Literal { val: 1 }],
            }),
          },
          ds,
        ),
      ],
    },
    ds,
  )];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![Cerr::ExpectedString(0, "test4".into()).with(ds)];
  assert_eq!(errs, expected);
}

#[test]
pub fn transform_err_unexpected_string() {
  let ds = Span::default();
  let ast = vec![
    (
      Module {
        name: "no_strings".into(),
        ports: vec![PortDecl {
          port_class: PortClass::In,
          signal_class: NetType::Single,
          name: "port".into(),
        }],
        stmts: vec![],
      },
      ds,
    ),
    (
      Module {
        name: "invalid_module".into(),
        ports: vec![],
        stmts: vec![
          (
            Stmt::WireDecl {
              name: "w1".into(),
              signal_class: NetType::Single,
              expr: None,
            },
            ds,
          ),
          (
            Stmt::WireDecl {
              name: "dst".into(),
              signal_class: NetType::Single,
              expr: None,
            },
            ds,
          ),
          (
            Stmt::Set {
              name: "dst".into(),
              assign_type: BinaryOp::AddAssign,
              expr: Expr::StringLiteral { str: "s".into() },
            },
            ds,
          ),
          (
            Stmt::Set {
              name: "dst".into(),
              assign_type: BinaryOp::AddAssign,
              expr: Expr::FnCall {
                func: "test3".into(),
                args: vec![Expr::StringLiteral { str: "s".into() }],
              },
            },
            ds,
          ),
          (
            Stmt::ModuleInst {
              module: "no_strings".into(),
              args: vec![Expr::StringLiteral { str: "s".into() }],
            },
            ds,
          ),
        ],
      },
      ds,
    ),
  ];
  let errs = transform_modules(&ast, &test_builtins()).1;
  let expected = vec![
    Cerr::UnexpectedString.with(ds),
    Cerr::UnexpectedString.with(ds),
    Cerr::UnexpectedString.with(ds),
  ];
  assert_eq!(errs, expected);
}
