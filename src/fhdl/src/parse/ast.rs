use crate::parse::tokenizer::BinaryOp;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Program {
  pub version: String,
  pub modules: Vec<Module>
}

impl Program {
  
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Module {
  pub name: String,
  pub ports: Vec<PortDecl>,
  pub stmts: Vec<Stmt>
}

impl Module {
  
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PortDecl {
  pub port_class: PortClass,
  pub signal_class: SignalClass,
  pub name: String,
}

impl PortDecl {
  
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum PortClass {
  In, Out, InOut,
}

impl PortClass {
  
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Stmt {
  MemDecl {
    name: String,
  },
  MemSet {
    name: String,
    add_assign: bool,
    expr: Expr,
  },
  WireDecl {
    name: String,
    expr: Expr,
  },
  ModuleInst {
    module: String,
    args: Vec<Expr>,
  },
  Trigger {
    wire: String,
    trigger_kind: TriggerKind,
    statements: Vec<Stmt>
  }
}

impl Stmt {
  
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TriggerKind {
  Increasing,
  Decreasing,
  Changed,
  Raw,
}

impl TriggerKind {
  
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SignalClass {
  Single,
  Mixed
}

impl SignalClass {
  
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Expr {
  Identifier {
    name: String,
  },
  Literal {
    val: i32,
  },
  FnCall {
    func: String,
    args: Vec<Expr>,
  },
  BinaryOps {
    car: Box<Expr>,
    cdr: Vec<(BinaryOp, Expr)>
  },
  Braced {
    inner: Box<Expr>
  },
}