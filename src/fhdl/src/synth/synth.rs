use crate::err::Cerr;
use crate::parse::ast::{NetType, PortDecl};
use crate::synth::combinator::{CCSignalRef, Combinator, Signal, SignalRef, SignalWithCount, VanillaCombinator, VanillaCombinatorOp};
use crate::synth::ir::{IRModule, IRModuleInst, IRStmt, IRTriggerStmt, IRValue, IRWireMemDecl};
use crate::synth::netlist::{Net, NetID, Netlist, WireColor};
use std::collections::{BTreeSet, HashMap};
use crate::synth::virt_signals::VIRTUAL_SIGNALS;

use super::builtins::{Builtins, SynthRef};

//todo better err reporting for synth errors

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SynthSettings {
  /// Sets which module should be synthesized.
  pub main: String,
  /// For the main module, sets the names of the ports.
  /// These will be displayed on the constant combinators used for
  /// module connections.
  pub main_module_conn_names: Vec<[char; 4]>,
  /// For the main module, sets the signals of single-signal ports.
  pub main_module_conn_signals: Vec<Signal>,
}

#[derive(Debug)]
pub struct GlobalSynthState<'a> {
  netlist: Netlist,
  collected_modules: HashMap<String, &'a IRModule>,
  builtin_functions: &'a Builtins,
}

impl<'a> GlobalSynthState<'a> {
  pub fn alloc_net(&mut self, ty: NetType, color: WireColor, signal: Option<Signal>) -> usize {
    let idx = self.netlist.nets.len();
    self.netlist.nets.push(Net {
      ty,
      color,
      signal,
      in_conn: vec![],
      out_conn: vec![],
    });
    idx
  }
}

pub type IncompleteNetID = usize;

/// The synthesis code needs to first map out all nets and dependencies between them
/// in order to allocate wire colours and signals. To do this, it creates
/// `IncompleteNet` instances, resolving colors and signals, then adding them to the netlist.
///
/// Notably, a single `IncompleteNet` might be synthesized to multiple real nets,
/// one red and one green wire.

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IncompleteNet {
  pub different_signal_as_net: Vec<IncompleteNetID>,
  pub different_color_as_net: Vec<IncompleteNetID>,
  pub resolved_signal: Option<Signal>,
  pub real_net: Option<(NetID, NetID)>,
  pub ty: NetType,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct IncompleteCombinator {
  c: Combinator,
  in1: Option<IncompleteNetID>,
  in2: Option<IncompleteNetID>,
  out: IncompleteNetID,
}

impl IncompleteCombinator {
  pub fn complete<'a>(self, mut f_get_net: impl FnMut(IncompleteNetID) -> (NetID, NetID), mut f_get_signal: impl FnMut(IncompleteNetID) -> &'a Option<Signal>) -> Combinator {
    let mut c = self.c;
    match &mut c {
      Combinator::Vanilla(comb2) => {
        // fix net references
        comb2.input_nets = [self.in1.map(|v2| f_get_net(v2).0), self.in2.map(|v2| f_get_net(v2).1)];
        comb2.output_nets = [Some(f_get_net(self.out).0), Some(f_get_net(self.out).1)];
        // fix signal references
        comb2.input_signals.iter_mut().for_each(|v| {
          if let SignalRef::IncompleteSignal(net_id) = v {
            *v = f_get_signal(*net_id).clone()
              .map(|v| SignalRef::Signal(v))
              .expect("Synth error: IncompleteCombinator refers signal of mixed net");
          }
        });
        if let SignalRef::IncompleteSignal(net_id) = comb2.output_signal {
          comb2.output_signal = f_get_signal(net_id).clone()
            .map(|v| SignalRef::Signal(v))
            .expect("Synth error: IncompleteCombinator refers signal of mixed net");
        }
      }
      Combinator::Constant(comb2) => {
        // fix net references
        comb2.output_nets = [self.in1.map(|v2| f_get_net(v2).0), self.in2.map(|v2| f_get_net(v2).1)];
        // fix signal references
        comb2.output_signals.iter_mut()
          .filter_map(|v| v.as_mut())
          .for_each(|v| {
            if let CCSignalRef::IncompleteSignal(net_id, val) = v {
              *v = f_get_signal(*net_id).clone()
                .map(|v2| CCSignalRef::Signal(SignalWithCount {
                  signal: v2,
                  count: *val
                }))
                .expect("Synth error: IncompleteCombinator refers signal of mixed net");
            }
          });
        
      }
    }
    c
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct IncompleteModule {
  module: String,
  args: Vec<IncompleteNetID>,
}

#[derive(Debug)]
pub struct ModuleSynthState<'a, 'b> {
  global_state: &'b mut GlobalSynthState<'a>,
  inc_nets: Vec<IncompleteNet>,
  inc_net_map: HashMap<String, IncompleteNetID>,
  inc_combinator: Vec<IncompleteCombinator>,
  inc_module: Vec<IncompleteModule>,
}

impl<'a, 'b> ModuleSynthState<'a, 'b> {
  pub fn new(global_state: &'b mut GlobalSynthState<'a>) -> Self {
    ModuleSynthState {
      global_state,
      inc_nets: vec![],
      inc_net_map: Default::default(),
      inc_combinator: vec![],
      inc_module: vec![],
    }
  }
  
  pub fn new_net_builder(&self) -> IncompleteNetBuilder {
    Default::default()
  }

  pub fn new_combinator(
    &mut self,
    c: Combinator,
    in1: Option<IncompleteNetID>,
    in2: Option<IncompleteNetID>,
    out: IncompleteNetID,
  ) {
    self
      .inc_combinator
      .push(IncompleteCombinator { c, in1, in2, out })
  }

  pub fn net_info(&self, id: IncompleteNetID) -> &IncompleteNet {
    &self.inc_nets[id]
  }
  
  pub fn find_net(&self, name: &str) -> Option<IncompleteNetID> {
    self.inc_net_map.get(name).cloned()
  }

  pub fn type_of(&self, r: &SynthRef) -> Option<NetType> {
    match r {
      SynthRef::Net(net) => Some(self.net_info(*net).ty),
      SynthRef::Value(_) => Some(NetType::Single),
      SynthRef::String(_) => None,
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct IncompleteNetBuilder {
  name: Option<String>,
  net_type: Option<NetType>,
  signal: Option<Signal>,
  real_net: Option<(NetID, NetID)>
}

impl IncompleteNetBuilder {
  pub fn new() -> Self {
    Default::default()
  }
  
  pub fn net_type(mut self, net_type: NetType) -> Self {
    self.net_type = Some(net_type);
    self
  }
  
  pub fn resolved_signal(mut self, signal: Signal) -> Self {
    self.signal = Some(signal);
    self
  }

  pub fn maybe_resolved_signal(mut self, signal: Option<Signal>) -> Self {
    self.signal = signal.or(self.signal);
    self
  }
  
  pub fn real_net(mut self, real_net: (NetID, NetID)) -> Self {
    self.real_net = Some(real_net);
    self
  }
  
  pub fn name(mut self, name: String) -> Self {
    self.name = Some(name);
    self
  }
  
  pub fn build(self, mod_state: &mut ModuleSynthState) -> IncompleteNetID {
    let id = mod_state.inc_nets.len();
    mod_state.inc_nets.push(IncompleteNet {
      different_signal_as_net: vec![],
      different_color_as_net: vec![],
      resolved_signal: self.signal,
      real_net: self.real_net,
      ty: self.net_type.expect("Net type not assigned"),
    });
    if let Some(name) = self.name {
      mod_state.inc_net_map.insert(name, id);
    }
    id
  }
}

pub fn synthesize(settings: &SynthSettings, modules: &[IRModule], builtins: &Builtins) -> Result<Netlist, Cerr> {
  let collected_modules = collect_modules(modules);
  let mut state = GlobalSynthState {
    netlist: Netlist {
      nets: vec![],
      net_external_conn: vec![],
      combinators: vec![],
    },
    collected_modules,
    builtin_functions: builtins,
  };
  // connect main module to the outside world
  let ports = &state
    .collected_modules
    .get(&settings.main)
    .ok_or(Cerr::MainNotFound(settings.main.clone()))?
    .ports;
  assert_eq!(ports.len(), settings.main_module_conn_names.len());
  assert_eq!(ports.len(), settings.main_module_conn_signals.len());
  let port_nets = ports
    .iter()
    .zip(settings.main_module_conn_signals.iter())
    .map(|(port, signal)| {
      if port.signal_class == NetType::Single {
        (
          state.alloc_net(port.signal_class, WireColor::Red, Some(signal.clone())),
          state.alloc_net(port.signal_class, WireColor::Green, Some(signal.clone())),
        )
      } else {
        (
          state.alloc_net(port.signal_class, WireColor::Red, None),
          state.alloc_net(port.signal_class, WireColor::Green, None),
        )
      }
    })
    .collect::<Vec<_>>();
  // now synthesize the main module
  synthesize_module(&mut state, &settings.main, &port_nets)?;
  Ok(state.netlist)
}

fn collect_modules(modules: &[IRModule]) -> HashMap<String, &IRModule> {
  let mut hashmap = HashMap::new();
  modules.iter().for_each(|module| {
    hashmap.insert(module.name.clone(), module);
  });
  hashmap
}

fn synthesize_module(
  state: &mut GlobalSynthState,
  name: &str,
  arg_nets: &[(NetID, NetID)],
) -> Result<(), Cerr> {
  let mut mod_state = ModuleSynthState::new(state);
  let module = *mod_state.global_state.collected_modules.get(name).unwrap();
  
  // collect module ports
  presynth_module_inputs(&mut mod_state, &module.ports, arg_nets);

  // collect wire mem decls
  presynth_ir_decls(&mut mod_state, &module.objects);
  
  // collect module inst
  presynth_modules(&mut mod_state, &module.module_inst);

  // run stmt presynth
  presynth_stmts(&mut mod_state, &module.stmts);
  
  // resolve signals
  resolve_signals(&mut mod_state);

  // convert all `IncompleteNet`s to real nets, and also synth inner modules
  complete_nets(mod_state);

  Ok(())
}

fn presynth_module_inputs(mod_state: &mut ModuleSynthState, port_decls: &[PortDecl], arg_nets: &[(NetID, NetID)]) -> Vec<IncompleteNetID> {
  arg_nets.iter()
    .enumerate()
    .map(|(i, &(r, g))| {
      mod_state.new_net_builder()
        .net_type(mod_state.global_state.netlist.nets[r].ty)
        .real_net((r, g))
        .name(port_decls[i].name.clone())
        .maybe_resolved_signal(mod_state.global_state.netlist.nets[r].signal.clone())
        .build(mod_state)
    })
    .collect()
}

fn presynth_ir_decls(mod_state: &mut ModuleSynthState, decls: &HashMap<String, IRWireMemDecl>) {
  decls.iter().for_each(|(name, decl)| {
    let net = mod_state.new_net_builder()
      .net_type(decl.ty)
      .name(name.clone())
      .build(mod_state);
    if decl.mem {
      mod_state.new_combinator(
        Combinator::Vanilla(VanillaCombinator {
          op: VanillaCombinatorOp::Eq,
          input_signals: [SignalRef::Const(0), SignalRef::Const(0)],
          output_signal: SignalRef::Anything,
          output_count: true,
          ..Default::default()
        }),
        Some(net),
        None,
        net,
      );
    }
  });
}

fn presynth_stmts(mod_state: &mut ModuleSynthState, stmts: &[IRStmt]) {
  stmts.iter()
    .for_each(|v| {
      let builtin = mod_state.global_state.builtin_functions.get(&v.op)
        .expect("Synth error: Invalid builtin");
      let inputs = v.args.iter()
        .map(|v| {
          match v {
            IRValue::Net(net) => SynthRef::Net(mod_state.find_net(net).expect("Synth error: Net not found")),
            IRValue::Lit(lit) => SynthRef::Value(*lit),
            IRValue::Str(str) => SynthRef::String(str.clone()),
          }
        })
        .collect::<Vec<_>>();
      let output = mod_state.find_net(&v.dest)
        .expect("Synth error: Net not found");
      builtin.synthesize(mod_state, &inputs, output)
        .expect("Synth error: Failed to synthesize builtin");
    });
}

fn presynth_trigger_stmt(mod_state: &mut ModuleSynthState, stmts: &[IRTriggerStmt]) {
  stmts.iter()
    .for_each(|v| {
      let src_net = mod_state.find_net(&v.src).expect("Synth error: Net not found");
      let dest_net = mod_state.find_net(&v.dest).expect("Synth error: Net not found");
      let on_net = mod_state.find_net(&v.on).expect("Synth error: Net not found");
      mod_state.new_combinator(Combinator::Vanilla(VanillaCombinator {
        op: VanillaCombinatorOp::Eq,
        input_signals: [SignalRef::IncompleteSignal(on_net), SignalRef::Const(0)],
        output_signal: SignalRef::Everything,
        output_count: true,
        .. Default::default()
      }), Some(src_net), Some(on_net), dest_net);
    });
}

fn presynth_modules(mod_state: &mut ModuleSynthState, modules: &[IRModuleInst]) {
  modules.iter()
    .for_each(|v| {
      mod_state.inc_module.push(IncompleteModule {
        module: v.name.clone(),
        args: v.args.iter().map(|v| {
          mod_state.find_net(v).expect("Synth error: Net not found")
        }).collect::<Vec<_>>(),
      })
    })
}

fn resolve_signals(mod_state: &mut ModuleSynthState) {
  let unresolved_list = mod_state.inc_nets
    .iter()
    .enumerate()
    .filter_map(|(id, net)| {
      if net.resolved_signal.is_none() && net.ty == NetType::Single {
        Some(id)
      } else {
        None
      }
    }).collect::<Vec<_>>();
  unresolved_list.into_iter().for_each(|id| {
    let net = &mod_state.inc_nets[id];
    let resolved_refs = net.different_signal_as_net.iter()
      .filter_map(|&v| {
        mod_state.inc_nets[v].resolved_signal.clone()
      })
      .collect::<BTreeSet<_>>();
    let virtual_signals = &VIRTUAL_SIGNALS;
    let resolved_signal = virtual_signals.iter().find(|v| !resolved_refs.contains(v));
    if let Some(resolved_signal) = resolved_signal {
      mod_state.inc_nets[id].resolved_signal = Some(resolved_signal.clone());
    } else {
      // no valid signals were found!
      panic!("Synth error: unable to allocate sufficient unique signals")
    }
  });
}

// consumes mod_state because it's the last operation and mod_state
// becomes de-facto invalid after this operation
fn complete_nets(mod_state: ModuleSynthState) {
  let state = mod_state.global_state;
  // expand each incomplete net into a netpair
  let completed_nets = mod_state.inc_nets
    .into_iter()
    .map(|v| {
      (state.alloc_net(v.ty, WireColor::Red, v.resolved_signal.clone()), state.alloc_net(v.ty, WireColor::Green, v.resolved_signal))
    })
    .collect::<Vec<_>>();
  // expand all combinators
  let f_get_net = |iid: usize| {
    completed_nets[iid]
  };
  let f_get_signal = |iid: usize| {
    &state.netlist.nets[completed_nets[iid].0].signal
  };
  let completed_combs = mod_state.inc_combinator.into_iter().map(|v| {
    let comb = v.complete(f_get_net, f_get_signal);
    let comb_id = state.netlist.combinators.len();
    state.netlist.combinators.push(comb);
    comb_id
  }).collect::<Vec<_>>();
  // add backrefs (net->combinator references)
  completed_combs.iter().for_each(|&cid| {
    let in_refs = match &state.netlist.combinators[cid] {
      Combinator::Vanilla(comb) => comb.input_nets.to_vec(),
      Combinator::Constant(comb) => vec![]
    };
    in_refs.into_iter()
      .enumerate()
      .filter_map(|(i, v)| v.map(|v| (i, v)))
      .for_each(|(i, v)| {
        state.netlist.nets[v].in_conn.push((cid, i));
      });
    let out_refs = match &state.netlist.combinators[cid] {
      Combinator::Vanilla(comb) => comb.output_nets,
      Combinator::Constant(comb) => comb.output_nets
    };
    out_refs.into_iter()
      .enumerate()
      .filter_map(|(i, v)| v.map(|v| (i, v)))
      .for_each(|(i, v)| {
        state.netlist.nets[v].out_conn.push((cid, i));
      });
  });
  // initialize submodules
  mod_state.inc_module.into_iter()
    .for_each(|v| {
      let arg_nets = v.args.into_iter()
        .map(|v| completed_nets[v])
        .collect::<Vec<_>>();
      synthesize_module(state, &v.module, &arg_nets)
        .expect("Synth error: submodule synth failed");
    });
  // done!
}
