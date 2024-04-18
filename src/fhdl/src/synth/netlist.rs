use crate::parse::ast::{NetType};
use crate::synth::combinator::Combinator;

/// Holds a synthesized netlist.
pub struct Netlist {
  pub nets: Vec<Net>,
  pub combinators: Vec<Combinator>,
}

pub struct Net {
  pub ty: NetType,
  pub color: WireColor,
  pub name: String,
  /// Named from the perspective of a combinator. This declares
  /// all combinators that read from the net.
  pub in_conn: Vec<(usize, usize)>,
  /// Named from the perspective of a combinator. This declares
  /// all combinators that write to the net.
  pub out_conn: Vec<(usize, usize)>,
}

pub enum WireColor {
  Red, Green
}

