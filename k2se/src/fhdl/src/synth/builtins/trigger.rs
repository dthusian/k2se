use crate::err::Cerr;
use crate::parse::ast::{NetType, TriggerKind};
use crate::synth::builtins::{BuiltinFunction, Builtins, FunctionArgReq, register, SynthRef};
use crate::synth::combinator::{Combinator, SignalRef, VanillaCombinator, VanillaCombinatorOp};
use crate::synth::synth::{IncompleteNetID, ModuleSynthState};

#[derive(Debug)]
pub struct TriggerFunc {
  trigger_mode: TriggerKind
}

impl TriggerFunc {
  pub fn collect(b: &mut Builtins) {
    register(b, "trig_inc", TriggerFunc { trigger_mode: TriggerKind::Increasing });
    register(b, "trig_dec", TriggerFunc { trigger_mode: TriggerKind::Decreasing });
    register(b, "trig_chg", TriggerFunc { trigger_mode: TriggerKind::Changed });
  }
}

impl BuiltinFunction for TriggerFunc {
  fn arg_ty(&self) -> &[FunctionArgReq] {
    &[FunctionArgReq::Net(NetType::Single)]
  }

  fn return_ty(&self) -> Option<NetType> {
    Some(NetType::Single)
  }

  fn synthesize(&self, state: &mut ModuleSynthState, inputs: &[SynthRef], output: IncompleteNetID) -> Result<(), Cerr> {
    let input_net = inputs[0].get_net().unwrap();
    let compare_op = match self.trigger_mode {
      TriggerKind::Increasing => VanillaCombinatorOp::Ge,
      TriggerKind::Decreasing => VanillaCombinatorOp::Le,
      TriggerKind::Changed => VanillaCombinatorOp::Eq,
      TriggerKind::Raw => unreachable!(),
    };
    // anonymous net that is the input but delayed
    let delayed_input = state.new_net_builder()
      .net_type(NetType::Single)
      .build(state);
    // passthrough combinator
    state.new_combinator(Combinator::Vanilla(VanillaCombinator {
      op: VanillaCombinatorOp::Add,
      input_signals: [SignalRef::Each, SignalRef::Const(0)],
      output_signal: SignalRef::Each,
      output_count: false,
      .. Default::default()
    }), Some(input_net), None, output);
    // compare combinator
    state.new_combinator(Combinator::Vanilla(VanillaCombinator {
      op: compare_op,
      input_signals: [SignalRef::IncompleteSignal(delayed_input), SignalRef::IncompleteSignal(input_net)],
      output_signal: SignalRef::IncompleteSignal(output),
      output_count: false,
      .. Default::default()
    }), Some(delayed_input), Some(input_net), output);
    Ok(())
  }

  fn constant_fold(&self, _: &[SynthRef]) -> Option<i32> {
    todo!()
  }
}