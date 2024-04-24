use std::collections::HashMap;
use crate::err::{Cerr, CerrSpan};
use crate::parse::ast::{Expr, Module, NetType, PortDecl, Stmt};
use crate::parse::span::Span;
use crate::synth::combinator::{Combinator, Signal, SignalRef, VanillaCombinator, VanillaCombinatorOp};
use crate::synth::netlist::{Net, NetID, Netlist, WireColor};

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

struct SynthState<'a> {
  netlist: Netlist,
  collected_modules: HashMap<String, (&'a Module, Span)>
}

impl<'a> SynthState<'a> {
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

pub fn synthesize(settings: &SynthSettings, modules: &[(Module, Span)]) -> Result<Netlist, CerrSpan> {
  let collected_modules = collect_modules(modules);
  let mut state = SynthState {
    netlist: Netlist {
      nets: vec![],
      combinators: vec![],
      combinator_modpath: vec![],
    },
    collected_modules,
  };
  let ports = &state.collected_modules
    .get(&settings.main)
    .ok_or(Cerr::MainNotFound(settings.main.clone()))?
    .0.ports;
  assert_eq!(ports.len(), settings.main_module_conn_names.len());
  assert_eq!(ports.len(), settings.main_module_conn_signals.len());
  let port_nets = ports.iter()
    .zip(settings.main_module_conn_signals.iter())
    .map(|(port, signal)| {
      if port.signal_class == NetType::Single {
        (state.alloc_net(port.signal_class, WireColor::Red, Some(signal.clone())),
        state.alloc_net(port.signal_class, WireColor::Green, Some(signal.clone())))
      } else {
        (state.alloc_net(port.signal_class, WireColor::Red, None),
        state.alloc_net(port.signal_class, WireColor::Green, None))
      }
    })
    .collect::<Vec<_>>();
  synthesize_module(&mut state, &settings.main, &port_nets)?;
  Ok(state.netlist)
}

fn collect_modules(modules: &[(Module, Span)]) -> HashMap<String, (&Module, Span)> {
  let mut hashmap = HashMap::new();
  modules.iter().for_each(|(module, span)| {
    hashmap.insert(module.name.clone(), (module, *span));
  });
  hashmap
}

type IncompleteNetID = usize;
type IncompleteNetPair = (IncompleteNetID, IncompleteNetID);
/// The synthesis code needs to first map out all nets and dependencies between them
/// in order to allocate wire colours and signals. To do this, it creates
/// `IncompleteNet` instances, resolving colors and signals, then adding them to the netlist.
struct IncompleteNet {
  different_signal_as_net: Vec<IncompleteNetID>,
  resolved_signal: Option<Signal>,
  color: WireColor,
  ty: NetType,
}

struct IncompleteCombinator {
  c: Combinator,
  in_r: Option<IncompleteNetID>,
  in_g: Option<IncompleteNetID>,
  out_r: Option<IncompleteNetID>,
  out_g: Option<IncompleteNetID>
}

struct IncompleteModule {
  module: String,
  args: Vec<IncompleteNetPair>
}

struct ModuleSynthState {
  inc_nets: Vec<IncompleteNet>,
  inc_net_map: HashMap<String, IncompleteNetPair>,
  inc_combinator: Vec<IncompleteCombinator>,
  inc_module: Vec<IncompleteModule>
}

impl ModuleSynthState {
  pub fn new_nets_resolved(&mut self, name: &str, net_type: NetType, resolved_signal: (Option<Signal>, Option<Signal>)) -> IncompleteNetPair {
    let new_net = self.new_nets_unnamed(net_type, resolved_signal);
    self.inc_net_map.insert(name.to_owned(), new_net);
    new_net
  }
  
  pub fn new_nets(&mut self, name: &str, net_type: NetType) -> IncompleteNetPair {
    self.new_nets_resolved(name, net_type, (None, None))
  }
  
  pub fn new_nets_unnamed(&mut self, net_type: NetType, resolved_signal: (Option<Signal>, Option<Signal>)) -> IncompleteNetPair {
    let ids = (self.inc_nets.len(), self.inc_nets.len() + 1);
    self.inc_nets.push(IncompleteNet {
      different_signal_as_net: vec![],
      resolved_signal: resolved_signal.0,
      color: WireColor::Red,
      ty: net_type,
    });
    self.inc_nets.push(IncompleteNet {
      different_signal_as_net: vec![],
      resolved_signal: resolved_signal.1,
      color: WireColor::Green,
      ty: net_type,
    });
    ids
  }
}

fn synthesize_module(state: &mut SynthState, name: &str, arg_nets: &[(NetID, NetID)]) -> Result<(), CerrSpan> {
  let mut mod_state = ModuleSynthState {
    inc_nets: vec![],
    inc_net_map: HashMap::new(),
    inc_combinator: vec![],
    inc_module: vec![],
  };
  let (module, _) = state.collected_modules.get(name).unwrap();
  let module = *module;
  
  // collect input module ports into incompletenet instances
  collect_ports_to_inc_nets(state, &mut mod_state, &module.ports, arg_nets);
  
  // collect wire mem decls
  presynth_wire_mem_decls(&mut mod_state, &module.stmts);
  
  // run expr presynth
  presynth_exprs(state, &mut mod_state, &module.stmts, None);
  
  // resolve signals
  resolve_signals(&mut mod_state);
  
  // convert all `IncompleteNet`s to real nets, and also synth inner modules
  complete_nets(mod_state);
  
  Ok(())
}

fn collect_ports_to_inc_nets(state: &mut SynthState, mod_state: &mut ModuleSynthState, ports: &[PortDecl], arg_nets: &[(NetID, NetID)]) {
  ports.iter()
    .zip(arg_nets.iter())
    .for_each(|(port, (net_id_r, net_id_g))| {
      let net_red = &state.netlist.nets[*net_id_r];
      let net_green = &state.netlist.nets[*net_id_g];
      mod_state.new_nets_resolved(&port.name, port.signal_class, (net_red.signal.clone(), net_green.signal.clone()));
    })
}

fn presynth_wire_mem_decls(mod_state: &mut ModuleSynthState, stmts: &[(Stmt, Span)]) {
  stmts.iter()
    .for_each(|(stmt, _)| {
      match stmt {
        Stmt::MemDecl { name, signal_class } => {
          let nets = mod_state.new_nets(name, *signal_class);
          mod_state.inc_combinator.push(IncompleteCombinator {
            c: Combinator::Vanilla(VanillaCombinator {
              op: VanillaCombinatorOp::Eq,
              input_nets: [None, None],
              output_nets: [None, None],
              input_signals: [SignalRef::Const(0), SignalRef::Const(0)],
              output_signal: SignalRef::Anything,
              output_count: true,
            }),
            in_r: Some(nets.0),
            in_g: None, // green is not self-fed because it would double-feed it
            out_r: Some(nets.0),
            out_g: Some(nets.1),
          })
        }
        Stmt::WireDecl { name, signal_class, .. } => {
          mod_state.new_nets(name, *signal_class);
        }
        _ => {}
      }
    });
}

fn presynth_exprs(state: &SynthState, mod_state: &mut ModuleSynthState, stmts: &[(Stmt, Span)], on_trigger: Option<IncompleteNetPair>) {
  stmts.iter()
    .for_each(|(stmt, _)| {
      match stmt {
        Stmt::MemDecl { .. } => {}
        Stmt::Set { name, expr, .. } => {
          let dest_nets = mod_state.inc_net_map[name];
          presynth_expr(mod_state, expr, dest_nets);
        }
        Stmt::WireDecl { name, expr, signal_class } => {
          let dest_nets = mod_state.inc_net_map[name];
          expr.as_ref().map(|v| {
            presynth_expr(mod_state, v, dest_nets);
          });
        }
        Stmt::ModuleInst { module, args } => {
          let arg_nets = args.iter().enumerate().map(|(i, v)| {
            match v {
              Expr::Identifier { name } => {
                mod_state.inc_net_map[name]
              }
              _ => {
                let module = state.collected_modules[module].0;
                let new_net = mod_state.new_nets_unnamed(module.ports[i].signal_class, (None, None));
                presynth_expr(mod_state, v, new_net);
                new_net
              }
            }
          }).collect::<Vec<_>>();
          mod_state.inc_module.push(IncompleteModule {
            module: module.clone(),
            args: arg_nets,
          })
        }
        Stmt::Trigger { watching, trigger_kind, statements } => {
          //todo synth the trigger stuff
          presynth_exprs(state, mod_state, statements, None); //todo change this to Some when trigger stuff is synthed
        }
      }
    });
}

fn presynth_expr(mod_state: &mut ModuleSynthState, expr: &Expr, assign_result_to: IncompleteNetPair) {
  todo!()
}

fn resolve_signals(mod_state: &mut ModuleSynthState) {
  todo!()
}

// consumes mod_state because it's the last operation and mod_state
// becomes de-facto invalid after this operation
fn complete_nets(mod_state: ModuleSynthState) {
  todo!()
}