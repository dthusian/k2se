use crate::parse::ast::{NetType};
use crate::synth::combinator::{Combinator, Signal};

pub type NetID = usize;
pub type CombinatorID = usize;

/// Holds a synthesized netlist.
pub struct Netlist {
  /// A list of nets.
  pub nets: Vec<Net>,
  /// A list of combinators that connect to the nets.
  pub combinators: Vec<Combinator>,
  /// Denotes through which modules a combinator was instantiated.
  /// Can be used by layout code to optimize placement. 
  pub combinator_modpath: Vec<Vec<usize>>
}

pub struct Net {
  pub ty: NetType,
  pub color: WireColor,
  pub signal: Option<Signal>,
  /// Named from the perspective of a combinator. This declares
  /// all combinators that read from the net.
  pub in_conn: Vec<(CombinatorID, usize)>,
  /// Named from the perspective of a combinator. This declares
  /// all combinators that write to the net.
  pub out_conn: Vec<(CombinatorID, usize)>,
}

pub enum WireColor {
  Red, Green
}

