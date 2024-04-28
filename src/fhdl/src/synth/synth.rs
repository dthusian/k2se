use crate::err::Cerr;
use crate::parse::ast::NetType;
use crate::synth::combinator::{
  Combinator, Signal, SignalRef, VanillaCombinator, VanillaCombinatorOp,
};
use crate::synth::ir::{IRModule, IRModuleInst, IRStmt, IRTriggerStmt, IRValue, IRWireMemDecl};
use crate::synth::netlist::{Net, NetID, Netlist, WireColor};
use std::collections::HashMap;

use super::builtins::{Builtins, SynthRef};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SynthSettings {
  /// Sets which module should be synthesized.
  main: String,
  /// For the main module, sets the names of the ports.
  /// These will be displayed on the constant combinators used for
  /// module connections.
  main_module_conn_names: Vec<[char; 4]>,
  /// For the main module, sets the signals of single-signal ports.
  main_module_conn_signals: Vec<Signal>,
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
/// Notably, a single `IncompleteNet` might be synthesized to multiple real nets
/// due to wire color allocations.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct IncompleteNet {
  pub different_signal_as_net: Vec<IncompleteNetID>,
  pub different_color_as_net: Vec<IncompleteNetID>,
  pub resolved_signal: Option<Signal>,
  pub ty: NetType,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct IncompleteCombinator {
  c: Combinator,
  in1: Option<IncompleteNetID>,
  in2: Option<IncompleteNetID>,
  out: IncompleteNetID,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct IncompleteModule {
  module: String,
  args: Vec<IncompleteNetID>,
}

#[derive(Debug)]
pub struct ModuleSynthState<'a, 'b> {
  global_state: &'b GlobalSynthState<'a>,
  inc_nets: Vec<IncompleteNet>,
  inc_net_map: HashMap<String, IncompleteNetID>,
  inc_combinator: Vec<IncompleteCombinator>,
  inc_module: Vec<IncompleteModule>,
}

impl<'a, 'b> ModuleSynthState<'a, 'b> {
  pub fn new_nets_resolved(
    &mut self,
    name: &str,
    net_type: NetType,
    resolved_signal: Option<Signal>,
  ) -> IncompleteNetID {
    let new_net = self.new_nets_unnamed_resolved(net_type, resolved_signal);
    self.inc_net_map.insert(name.to_owned(), new_net);
    new_net
  }

  pub fn new_nets(&mut self, name: &str, net_type: NetType) -> IncompleteNetID {
    self.new_nets_resolved(name, net_type, None)
  }

  pub fn new_nets_unnamed_resolved(
    &mut self,
    net_type: NetType,
    resolved_signal: Option<Signal>,
  ) -> IncompleteNetID {
    let id = self.inc_nets.len();
    self.inc_nets.push(IncompleteNet {
      different_signal_as_net: vec![],
      different_color_as_net: vec![],
      resolved_signal,
      ty: net_type,
    });
    id
  }

  pub fn new_nets_unnamed(&mut self, net_type: NetType) -> IncompleteNetID {
    self.new_nets_unnamed_resolved(net_type, None)
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

pub fn synthesize(settings: &SynthSettings, modules: &[IRModule], builtins: &Builtins) -> Result<Netlist, Cerr> {
  let collected_modules = collect_modules(modules);
  let mut state = GlobalSynthState {
    netlist: Netlist {
      nets: vec![],
      net_external_conn: vec![],
      combinators: vec![],
      combinator_modpath: vec![],
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
  let mut mod_state = ModuleSynthState {
    global_state: &state,
    inc_nets: vec![],
    inc_net_map: HashMap::new(),
    inc_combinator: vec![],
    inc_module: vec![],
  };
  let module = *state.collected_modules.get(name).unwrap();

  // collect wire mem decls
  presynth_ir_decls(&mut mod_state, &module.objects);
  
  // collect module inst
  presynth_modules(&mut mod_state, &module.module_inst);

  // run stmt presynth
  presynth_stmts(&mut mod_state, &module.stmts);

  // run trigger stmt presynth
  
  
  // resolve signals
  resolve_signals(&mut mod_state);

  // convert all `IncompleteNet`s to real nets, and also synth inner modules
  complete_nets(mod_state);

  Ok(())
}

fn presynth_ir_decls(mod_state: &mut ModuleSynthState, decls: &HashMap<String, IRWireMemDecl>) {
  decls.iter().for_each(|(name, decl)| {
    let net = mod_state.new_nets(name, decl.ty);
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
  todo!()
}

// consumes mod_state because it's the last operation and mod_state
// becomes de-facto invalid after this operation
fn complete_nets(mod_state: ModuleSynthState) {
  todo!()
}
