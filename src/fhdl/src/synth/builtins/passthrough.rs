use crate::err::Cerr;
use crate::parse::ast::NetType;
use crate::synth::builtins::{BuiltinFunction, Builtins, FunctionArgReq, SynthRef};
use crate::synth::combinator::{CCSignalRef, Combinator, ConstantCombinator, SignalRef, VanillaCombinator, VanillaCombinatorOp};
use crate::synth::synth::{IncompleteNetID, ModuleSynthState};

#[derive(Debug)]
pub struct Passthrough;

impl Passthrough {
  pub fn collect(b: &mut Builtins) {
    b.insert("$passthrough".into(), Box::new(Passthrough));
  }
}

impl BuiltinFunction for Passthrough {
  fn arg_ty(&self) -> &[FunctionArgReq] {
    &[FunctionArgReq::Any]
  }

  fn return_ty(&self) -> Option<NetType> {
    None
  }

  fn synthesize(&self, state: &mut ModuleSynthState, inputs: &[SynthRef], output: IncompleteNetID) -> Result<(), Cerr> {
    let input = &inputs[0];
    match input {
      SynthRef::Net(net) => {
        if state.net_info(*net).ty == NetType::Mixed {
          state.new_combinator(Combinator::Vanilla(VanillaCombinator {
            op: VanillaCombinatorOp::Add,
            input_nets: [None, None],
            output_nets: [None, None],
            input_signals: [SignalRef::Each, SignalRef::Const(0)],
            output_signal: SignalRef::Each,
            output_count: false,
          }), Some(*net), None, output);
        } else {
          state.new_combinator(Combinator::Vanilla(VanillaCombinator {
            op: VanillaCombinatorOp::Add,
            input_nets: [None, None],
            output_nets: [None, None],
            input_signals: [SignalRef::IncompleteSignal(*net), SignalRef::Const(0)],
            output_signal: SignalRef::IncompleteSignal(output),
            output_count: false,
          }), Some(*net), None, output);
        }
      }
      SynthRef::Value(val) => {
        if state.net_info(output).ty == NetType::Mixed {
          
        }
        state.new_combinator(Combinator::Constant(ConstantCombinator {
          enabled: true,
          output_nets: [None, None],
          output_signals: vec![CCSignalRef::IncompleteSignal(output, *val)],
        }), None, None, output);
      }
      SynthRef::String(_) => panic!("Unexpected string")
    }
    
    Ok(())
  }

  fn constant_fold(&self, _: &[SynthRef]) -> Option<i32> {
    todo!()
  }
}