//! Before modules are synthesized, they are transformed to a simplified IR.
//! In this IR, wire declarations are hoisted, expressions are flattenned,
//! trigger blocks are refactored to only contain set statements, and trigger
//! conditions are synthesized. Additionally, type checking is done.

use std::collections::HashMap;
use crate::parse::ast::{NetType};

pub struct IRModule {
  pub objects: HashMap<String, IRWireMemDecl>,
  pub stmts: Vec<IRStmt>,
  pub trigger_stmt: Vec<IRTriggerStmt>,
  pub module_inst: Vec<IRModuleInst>
}

pub struct IRWireMemDecl {
  pub ty: NetType,
  pub mem: bool,
  pub port_idx: Option<usize>
}

pub struct IRStmt {
  pub dest: String,
  pub op: String,
  pub args: Vec<IRNetOrLiteral>
}

pub enum IRNetOrLiteral {
  Net(String),
  Lit(i32)
}

pub struct IRTriggerStmt {
  pub dest: String,
  pub src: String,
  pub on: String,
}

pub struct IRModuleInst {
  pub name: String,
  pub args: Vec<String>
}