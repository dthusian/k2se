pub mod binaryop;
mod trigger;

use crate::err::Cerr;
use crate::parse::ast::NetType;
use crate::synth::builtins::binaryop::BinaryOpFunc;
use crate::synth::synth::{IncompleteNetID, ModuleSynthState};
use std::collections::HashMap;
use crate::synth::combinator::{SignalRef};

/// Defines the requirements placed on a function argument during type checking.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FunctionArgReq {
  Any,
  Net(NetType),
  SingleOrLit,
  String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SynthRef {
  Net(IncompleteNetID),
  Value(i32),
  String(String),
}

impl SynthRef {
  pub fn get_net(&self) -> Option<IncompleteNetID> {
    match self {
      SynthRef::Net(net) => Some(*net),
      _ => None,
    }
  }
  
  pub fn as_signal_ref(&self) -> Option<SignalRef> {
    match self {
      SynthRef::Net(net) => Some(SignalRef::IncompleteSignal(*net)),
      SynthRef::Value(v) => Some(SignalRef::Const(*v)),
      SynthRef::String(_) => None
    }
  }
}

/// Trait implemented for each built-in function.
pub trait BuiltinFunction {
  fn arg_ty(&self) -> &[FunctionArgReq];
  fn return_ty(&self) -> NetType;
  fn synthesize(&self, state: &mut ModuleSynthState, inputs: &[SynthRef], output: IncompleteNetID) -> Result<(), Cerr>;
  fn constant_fold(&self, args: &[SynthRef]) -> Option<i32>;
}

type Builtins = HashMap<String, Box<dyn BuiltinFunction>>;

pub fn collect_builtins() -> Builtins {
  let mut b = Builtins::new();
  BinaryOpFunc::collect(&mut b);
  b
}

fn register<T: BuiltinFunction + 'static>(b: &mut Builtins, name: &str, t: T) {
  b.insert(name.into(), Box::new(t));
}
