//! Before modules are synthesized, they are transformed to a simplified IR.
//! In this IR, wire declarations are hoisted, expressions are flattenned,
//! trigger blocks are refactored to only contain set statements, and trigger
//! conditions are synthesized. Additionally, type checking is done.

use crate::parse::ast::NetType;

pub struct IRModule {
  pub wire_decls: Vec<IRWireMemDecl>,
  pub mem_decls: Vec<IRWireMemDecl>,
  pub stmts: Vec<IRStmt>,
  pub triggers: Vec<IRStmt>
}

pub struct IRWireMemDecl {
  pub name: String,
  pub ty: NetType
}

pub struct IRStmt {
  pub dest: String,
  pub op: String,
  pub args: Vec<String>
}
