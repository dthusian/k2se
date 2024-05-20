use std::collections::{BTreeSet, VecDeque};
use crate::layout::location_searcher::LocationSearcher;
use crate::layout::shapers::power_pole_shaper::PowerPoleShaper;
use crate::synth::combinator::Combinator;
use crate::synth::netlist::Netlist;

pub enum WireConnectionPoint {
  RedIn = 0,
  GreenIn = 1,
  RedOut = 2,
  GreenOut = 3,
}

pub struct Layout {
  pub entity_list: Vec<LayoutEntity>
}

pub struct LayoutEntity {
  pub pos: (i32, i32),
  pub combinator: Combinator,
  pub connections: [(usize, WireConnectionPoint); 4]
}

struct LayoutState {
  loc: LocationSearcher
}

pub fn make_layout(netlist: Netlist) {
  let state = LayoutState {
    loc: LocationSearcher::new(Box::new(PowerPoleShaper::default())),
  };
  
}
