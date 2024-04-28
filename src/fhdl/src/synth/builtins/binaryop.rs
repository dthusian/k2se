use crate::err::Cerr;
use crate::parse::ast::NetType;
use crate::parse::tokenizer::BinaryOp;
use crate::synth::builtins::{register, BuiltinFunction, Builtins, FunctionArgReq, SynthRef};
use crate::synth::combinator::{Combinator, SignalRef, VanillaCombinator, VanillaCombinatorOp};
use crate::synth::synth::{IncompleteNetID, ModuleSynthState};

/// Builtin functions for binary ops.
/// Unlike most builtin functions, these cannot be referred to by users directly
/// and are synthesized directly by the transformation stage. As a result,
/// they bypass typechecking and have ill-defined input/output types.
#[derive(Debug)]
pub struct BinaryOpFunc {
  op: BinaryOp,
}

impl BinaryOpFunc {
  fn register(reg: &mut Builtins, op: BinaryOp) {
    register(reg, binary_op_to_func_name(op), BinaryOpFunc { op });
  }

  pub fn collect(reg: &mut Builtins) {
    Self::register(reg, BinaryOp::Add);
    Self::register(reg, BinaryOp::Sub);
    Self::register(reg, BinaryOp::Mul);
    Self::register(reg, BinaryOp::Div);
    Self::register(reg, BinaryOp::Mod);
    Self::register(reg, BinaryOp::Pow);
    Self::register(reg, BinaryOp::And);
    Self::register(reg, BinaryOp::Or);
    Self::register(reg, BinaryOp::Xor);
    Self::register(reg, BinaryOp::Shl);
    Self::register(reg, BinaryOp::Shr);
    Self::register(reg, BinaryOp::Eq);
    Self::register(reg, BinaryOp::Ne);
    Self::register(reg, BinaryOp::Lt);
    Self::register(reg, BinaryOp::Gt);
    Self::register(reg, BinaryOp::Le);
    Self::register(reg, BinaryOp::Ge);
  }
}

impl BuiltinFunction for BinaryOpFunc {
  fn arg_ty(&self) -> &[FunctionArgReq] {
    panic!("binary ops bypass typechecking")
  }

  fn return_ty(&self) -> NetType {
    panic!("binary ops bypass typechecking")
  }

  fn synthesize(
    &self,
    state: &mut ModuleSynthState,
    inputs: &[SynthRef],
    output: IncompleteNetID,
  ) -> Result<(), Cerr> {
    // strings should not exist here
    let ty1 = state.type_of(&inputs[0]).unwrap();
    let ty2 = state.type_of(&inputs[1]).unwrap();
    if ty1 == NetType::Mixed && ty2 == NetType::Mixed {
      // case 1: both mixed
      // only nets can be mixed
      let net1 = inputs[0].get_net().unwrap();
      let net2 = inputs[1].get_net().unwrap();
      let net2 = if self.op == BinaryOp::Add {
        net2
      } else if self.op == BinaryOp::Sub {
        let anon = state.new_nets_unnamed(NetType::Mixed);
        state.new_combinator(Combinator::Vanilla(VanillaCombinator {
          op: VanillaCombinatorOp::Mul,
          input_signals: [SignalRef::Each, SignalRef::Const(-1)],
          output_signal: SignalRef::Anything,
          output_count: false,
          .. Default::default()
        }), Some(net2), None, anon);
        anon
      } else {
        panic!("Invalid use of mixed nets")
      };
      state.new_combinator(Combinator::Vanilla(VanillaCombinator {
        op: VanillaCombinatorOp::Add,
        input_signals: [SignalRef::Each, SignalRef::Const(0)],
        output_signal: SignalRef::Each,
        output_count: false,
        .. Default::default()
      }), Some(net1), Some(net2), output);
    } else if ty1 == NetType::Mixed || ty2 == NetType::Mixed {
      // case 2: single-mixed op
      // warning: single-mixed ops are "dirty": they leak virtual signals into the output
      let (mixed_net, single) = if ty1 == NetType::Mixed {
        (inputs[0].get_net().unwrap(), &inputs[1])
      } else {
        (inputs[1].get_net().unwrap(), &inputs[0])
      };
      state.new_combinator(Combinator::Vanilla(VanillaCombinator {
        // unwrap: assign type ops are forbidden in expressions
        op: self.op.try_into().unwrap(),
        input_signals: [SignalRef::Each, single.as_signal_ref().unwrap()],
        output_signal: SignalRef::Each,
        output_count: false,
        .. Default::default()
      }), Some(mixed_net), single.get_net(), output);
    } else {
      // case3: single-single op
      // this is relatively simple
      state.new_combinator(Combinator::Vanilla(VanillaCombinator {
        // unwrap: assign type ops are forbidden in expressions
        op: self.op.try_into().unwrap(),
        input_signals: [inputs[0].as_signal_ref().unwrap(), inputs[1].as_signal_ref().unwrap()],
        output_signal: SignalRef::IncompleteSignal(output),
        output_count: false,
        .. Default::default()
      }), inputs[0].get_net(), inputs[1].get_net(), output);
    }
    Ok(())
  }

  fn constant_fold(&self, _: &[SynthRef]) -> Option<i32> {
    None
  }
}

pub fn binary_op_to_func_name(op: BinaryOp) -> &'static str {
  match op {
    BinaryOp::Add => "$op_add",
    BinaryOp::Sub => "$op_sub",
    BinaryOp::Mul => "$op_mul",
    BinaryOp::Div => "$op_div",
    BinaryOp::Mod => "$op_mod",
    BinaryOp::Pow => "$op_pow",
    BinaryOp::And => "$op_and",
    BinaryOp::Or => "$op_or",
    BinaryOp::Xor => "$op_xor",
    BinaryOp::Shl => "$op_shl",
    BinaryOp::Shr => "$op_shr",
    BinaryOp::Eq => "$op_eq",
    BinaryOp::Ne => "$op_ne",
    BinaryOp::Lt => "$op_lt",
    BinaryOp::Gt => "$op_gt",
    BinaryOp::Le => "$op_le",
    BinaryOp::Ge => "$op_ge",
    _ => panic!("Invalid BinaryOp"),
  }
}
