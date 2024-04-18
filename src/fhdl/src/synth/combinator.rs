use crate::parse::tokenizer::BinaryOp;

/// Represents any combinator or circuit computational entity.
/// If there's a 2-array of input nets, the convention is `[red, green]`.
pub enum Combinator {
  /// Represents an arithmetic or decider combinator.
  Vanilla(VanillaCombinator),
  /// Represents a constant combinator.
  Constant(ConstantCombinator)
}

/// Represents an arithmetic or decider combinator.
pub struct VanillaCombinator {
  pub op: VanillaCombinatorOp,
  pub input_nets: [Option<usize>; 2],
  pub output_nets: [Option<usize>; 2],
  pub input_signals: [SignalRef; 2],
  pub output_signal: SignalRef,
  /// Refers to whether the "input count" setting is enabled on a decider combinator.
  pub output_count: bool,
}

/// Represents a constant combinator.
pub struct ConstantCombinator {
  pub output_nets: [Option<usize>; 2],
  pub output_signals: [Option<SignalWithCount>; 20]
}

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
  Le
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

pub enum SignalType {
  Item, Fluid, Virtual
}

pub struct Signal {
  pub ty: SignalType,
  pub name: String,
}

pub struct SignalWithCount {
  pub signal: Signal,
  pub count: i32,
}

pub enum SignalRef {
  Anything,
  Each,
  Everything,
  Signal(Signal),
  Const(i32)
}