use crate::parse::ast::NetType;
use crate::synth::combinator::{Combinator, Signal};

pub type NetID = usize;
pub type CombinatorID = usize;

/// Holds a synthesized netlist.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Netlist {
  /// A list of nets.
  pub nets: Vec<Net>,
  /// A list of external connections.
  pub net_external_conn: Vec<ExternalConn>,
  /// A list of combinators that connect to the nets.
  pub combinators: Vec<Combinator>,
  /// Denotes through which modules a combinator was instantiated.
  /// Can be used by layout code to optimize placement.
  pub combinator_modpath: Vec<Vec<usize>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExternalConn {
  pub red_net: NetID,
  pub green_net: NetID,
  pub name: [char; 4],
  pub signal: Option<Signal>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WireColor {
  Red,
  Green,
}
