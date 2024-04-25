use crate::parse::tokenizer::BinaryOp;
use crate::synth::netlist::NetID;
use crate::synth::synth::IncompleteNetID;

/// Represents any combinator or circuit computational entity.
/// If there's a 2-array of input nets, the convention is `[red, green]`.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Combinator {
  /// Represents an arithmetic or decider combinator.
  Vanilla(VanillaCombinator),
  /// Represents a constant combinator.
  Constant(ConstantCombinator),
}

/// Represents an arithmetic or decider combinator.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VanillaCombinator {
  pub op: VanillaCombinatorOp,
  pub input_nets: [Option<NetID>; 2],
  pub output_nets: [Option<NetID>; 2],
  pub input_signals: [SignalRef; 2],
  pub output_signal: SignalRef,
  /// Refers to whether the "input count" setting is enabled on a decider combinator.
  pub output_count: bool,
}

impl Default for VanillaCombinator {
  fn default() -> Self {
    VanillaCombinator {
      op: VanillaCombinatorOp::Add,
      input_nets: [None, None],
      output_nets: [None, None],
      input_signals: [SignalRef::Each, SignalRef::Const(0)],
      output_signal: SignalRef::Each,
      output_count: false,
    }
  }
}

/// Represents a constant combinator.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ConstantCombinator {
  pub enabled: bool,
  pub output_nets: [Option<NetID>; 2],
  pub output_signals: [Option<SignalWithCount>; 20],
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum VanillaCombinatorOp {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Pow,
  And,
  Or,
  Xor,
  Shl,
  Shr,
  Eq,
  Ne,
  Gt,
  Lt,
  Ge,
  Le,
}

impl TryFrom<BinaryOp> for VanillaCombinatorOp {
  type Error = ();

  fn try_from(value: BinaryOp) -> Result<Self, Self::Error> {
    Ok(match value {
      BinaryOp::Add => VanillaCombinatorOp::Add,
      BinaryOp::Sub => VanillaCombinatorOp::Sub,
      BinaryOp::Mul => VanillaCombinatorOp::Mul,
      BinaryOp::Div => VanillaCombinatorOp::Div,
      BinaryOp::Mod => VanillaCombinatorOp::Mod,
      BinaryOp::Pow => VanillaCombinatorOp::Pow,
      BinaryOp::And => VanillaCombinatorOp::And,
      BinaryOp::Or => VanillaCombinatorOp::Or,
      BinaryOp::Xor => VanillaCombinatorOp::Xor,
      BinaryOp::Shl => VanillaCombinatorOp::Shl,
      BinaryOp::Shr => VanillaCombinatorOp::Shr,
      BinaryOp::Eq => VanillaCombinatorOp::Eq,
      BinaryOp::Ne => VanillaCombinatorOp::Ne,
      BinaryOp::Lt => VanillaCombinatorOp::Lt,
      BinaryOp::Gt => VanillaCombinatorOp::Gt,
      BinaryOp::Le => VanillaCombinatorOp::Le,
      BinaryOp::Ge => VanillaCombinatorOp::Ge,
      BinaryOp::Assign => return Err(()),
      BinaryOp::AddAssign => return Err(()),
    })
  }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SignalType {
  Item,
  Fluid,
  Virtual,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Signal {
  pub ty: SignalType,
  pub name: String,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SignalWithCount {
  pub signal: Signal,
  pub count: i32,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SignalRef {
  Anything,
  Each,
  Everything,
  Signal(Signal),
  IncompleteSignal(IncompleteNetID),
  Const(i32),
}
