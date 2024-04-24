use std::collections::HashMap;
use crate::parse::ast::NetType;
use crate::synth::netlist::NetID;

pub enum FunctionArgReq {
  Any,
  NetType(NetType),
  SingleOrLit,
  String
}

pub trait BuiltinFunction {
  fn arg_ty(&self) -> &[FunctionArgReq];
  fn return_ty(&self) -> NetType;
  fn synthesize(&self, inputs: &[(NetID, NetID)], output: (NetID, NetID));
  fn constant_fold(&self, args: &[i32]) -> Option<i32>;
}

pub fn collect_builtins() -> HashMap<String, Box<dyn BuiltinFunction>> {
  todo!()
}