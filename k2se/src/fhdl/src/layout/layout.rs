use std::collections::{BTreeSet, VecDeque};
use crate::layout::location_searcher::LocationSearcher;
use crate::layout::shapers::power_pole_shaper::PowerPoleShaper;
use crate::synth::combinator::Combinator;
use crate::synth::netlist::Netlist;

struct LayoutState {
  loc: LocationSearcher
}

pub fn make_layout(netlist: Netlist) {
  let state = LayoutState {
    loc: LocationSearcher::new(Box::new(PowerPoleShaper::default())),
  };
  
}
