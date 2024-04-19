use crate::err::Cerr;
use crate::parse::ast::{Expr, Module, NetType, PortClass, PortDecl, Stmt, TriggerKind};
use crate::parse::span::Span;
use crate::parse::tokenizer::BinaryOp;
use crate::synth::validate::validate_modules;

#[test]
pub fn validate_undeclared() {
  let ds = Span::default();
  let ast = vec![(Module {
    name: "invalid_module".into(),
    ports: vec![
      PortDecl {
        port_class: PortClass::Out,
        signal_class: NetType::Single,
        name: "port".into(),
      }
    ],
    stmts: vec![
      (Stmt::WireDecl {
        name: "wire1".into(),
        signal_class: NetType::Single,
        expr: Some(Expr::BinaryOps {
          car: Box::new(Expr::Identifier { name: "undeclared1".into() }),
          cdr: vec![(BinaryOp::Add, Expr::Literal { val: 2 })],
        }),
      }, ds),
      (Stmt::Set {
        name: "port".into(),
        assign_type: BinaryOp::Assign,
        expr: Expr::Identifier { name: "undeclared2".into() },
      }, ds),
      (Stmt::Trigger {
        watching: "undeclared3".into(),
        trigger_kind: TriggerKind::Increasing,
        statements: vec![],
      }, ds),
      (Stmt::ModuleInst {
        module: "undeclared4".into(),
        args: vec![],
      }, ds),
    ],
  }, ds)];
  let errs = validate_modules(&ast);
  let expected = vec![
    Cerr::NotDeclared("undeclared1".into()).with(ds),
    Cerr::NotDeclared("undeclared2".into()).with(ds),
    Cerr::NotDeclared("undeclared3".into()).with(ds),
    Cerr::NotDeclared("undeclared4".into()).with(ds),
  ];
  assert_eq!(errs, expected);
}

#[test]
pub fn validate_multiple_decl() {
  let ds = Span::default();
  let ast = vec![
    (Module {
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
        }
      ],
      stmts: vec![],
    }, ds),
    (Module {
      name: "multi1".into(),
      ports: vec![],
      stmts: vec![
        (Stmt::WireDecl {
          name: "multi4".into(),
          signal_class: NetType::Single,
          expr: None,
        }, ds),
        (Stmt::MemDecl {
          name: "multi4".into(),
          signal_class: NetType::Single,
        }, ds),
        (Stmt::WireDecl {
          name: "multi3".into(),
          signal_class: NetType::Single,
          expr: None,
        }, ds),
        (Stmt::WireDecl {
          name: "multi3".into(),
          signal_class: NetType::Single,
          expr: None,
        }, ds),
        (Stmt::MemDecl {
          name: "multi5".into(),
          signal_class: NetType::Single,
        }, ds),
        (Stmt::MemDecl {
          name: "multi5".into(),
          signal_class: NetType::Single,
        }, ds),
      ],
    }, ds)
  ];
  let errs = validate_modules(&ast);
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
pub fn validate_writes_to_input() {
  let ds = Span::default();
  let ast = vec![(Module {
    name: "invalid_module".into(),
    ports: vec![
      PortDecl {
        port_class: PortClass::In,
        signal_class: NetType::Single,
        name: "inputter".into(),
      }
    ],
    stmts: vec![
      (Stmt::Set {
        name: "inputter".into(),
        assign_type: BinaryOp::AddAssign,
        expr: Expr::Literal { val: 5 },
      }, ds)
    ],
  }, ds)];
  let errs = validate_modules(&ast);
  let expected = vec![
    Cerr::WriteToInput.with(ds)
  ];
  assert_eq!(errs, expected);
}

#[test]
pub fn validate_multiple_excl_assign() {
  let ds = Span::default();
  let ast = vec![(Module {
    name: "invalid_module".into(),
    ports: vec![],
    stmts: vec![
      (Stmt::WireDecl {
        name: "w1".into(),
        signal_class: NetType::Single,
        expr: None,
      }, ds),
      (Stmt::Set {
        name: "w1".into(),
        assign_type: BinaryOp::Assign,
        expr: Expr::Literal { val: 2 },
      }, ds),
      (Stmt::Set {
        name: "w1".into(),
        assign_type: BinaryOp::Assign,
        expr: Expr::Literal { val: 2 },
      }, ds),
      (Stmt::WireDecl {
        name: "w2".into(),
        signal_class: NetType::Single,
        expr: Some(Expr::Literal { val: 4 }),
      }, ds),
      (Stmt::Set {
        name: "w2".into(),
        assign_type: BinaryOp::AddAssign,
        expr: Expr::Literal { val: 4 },
      }, ds)
    ],
  }, ds)];
  let errs = validate_modules(&ast);
  let expected = vec![
    Cerr::MultipleExclusiveWrites.with(ds),
    Cerr::MultipleExclusiveWrites.with(ds)
  ];
  assert_eq!(errs, expected);
}

#[test]
pub fn validate_nested_trigger() {
  let ds = Span::default();
  let ast = vec![(Module {
    name: "invalid_module".into(),
    ports: vec![
      PortDecl {
        port_class: PortClass::In,
        signal_class: NetType::Single,
        name: "w1".into(),
      }
    ],
    stmts: vec![
      (Stmt::Trigger {
        watching: "w1".into(),
        trigger_kind: TriggerKind::Raw,
        statements: vec![
          (Stmt::Trigger {
            watching: "w1".to_string(),
            trigger_kind: TriggerKind::Raw,
            statements: vec![],
          }, ds)
        ],
      }, ds)
    ],
  }, ds)];
  let errs = validate_modules(&ast);
  let expected = vec![
    Cerr::NestedTriggerBlocks.with(ds)
  ];
  assert_eq!(errs, expected);
}

#[test]
pub fn validate_bare_mem_assign() {
  let ds = Span::default();
  let ast = vec![(Module {
    name: "invalid_module".into(),
    ports: vec![],
    stmts: vec![
      (Stmt::MemDecl {
        name: "reg".into(),
        signal_class: NetType::Single,
      }, ds),
      (Stmt::Set {
        name: "reg".into(),
        assign_type: BinaryOp::Assign,
        expr: Expr::Literal { val: 9 },
      }, ds),
    ],
  }, ds)];
  let errs = validate_modules(&ast);
  let expected = vec![
    Cerr::MemAssignOutsideOfTrigger.with(ds)
  ];
  assert_eq!(errs, expected);
}

#[test]
pub fn validate_arity_mismatch() {
  let ds = Span::default();
  let ast = vec![
    (Module {
      name: "target_module".into(),
      ports: vec![
        PortDecl {
          port_class: PortClass::In,
          signal_class: NetType::Single,
          name: "p1".to_string(),
        },
        PortDecl {
          port_class: PortClass::Out,
          signal_class: NetType::Single,
          name: "p2".to_string(),
        },
      ],
      stmts: vec![],
    }, ds),
    (Module {
      name: "invalid_module".into(),
      ports: vec![],
      stmts: vec![
        (Stmt::WireDecl {
          name: "w1".into(),
          signal_class: NetType::Single,
          expr: None,
        }, ds),
        (Stmt::ModuleInst {
          module: "target_module".into(),
          args: vec![
            Expr::Identifier { name: "w1".into() },
            Expr::Identifier { name: "w1".into() },
            Expr::Identifier { name: "w1".into() },
          ],
        }, ds),
      ],
    }, ds),
  ];
  let errs = validate_modules(&ast);
  let expected = vec![
    Cerr::WrongNumberOfModuleArgs(2).with(ds)
  ];
  assert_eq!(errs, expected);
}