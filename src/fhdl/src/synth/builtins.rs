use crate::parse::ast::NetType;
use crate::synth::netlist::NetID;

pub trait BuiltinFunction {
  fn arg_ty(&self) -> &[NetType];
  fn return_ty(&self) -> NetType;
  fn synthesize(&self, inputs: &[(NetID, NetID)], output: (NetID, NetID));
}

pub fn collect_builtins() -> Vec<Box<dyn BuiltinFunction>> {
  todo!()
}