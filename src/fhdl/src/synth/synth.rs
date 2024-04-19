use std::collections::HashMap;
use crate::err::{Cerr, CerrSpan};
use crate::parse::ast::{Expr, Module, NetType, PortDecl, Stmt};
use crate::parse::span::Span;
use crate::synth::combinator::Signal;
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
        state.alloc_net(port.signal_class, WireColor::Red, Some(signal.clone()))
      } else {
        state.alloc_net(port.signal_class, WireColor::Red, None)
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
/// The synthesis code needs to first map out all nets and dependencies between them
/// in order to allocate wire colours and signals. To do this, it creates
/// `IncompleteNet` instances, resolving colors and signals, then adding them to the netlist.
struct IncompleteNet {
  different_color_as_net: Vec<IncompleteNetID>,
  different_signal_as_net: Vec<IncompleteNetID>,
  resolved_color: Option<WireColor>,
  resolved_signal: Option<Signal>,
  ty: NetType,
}

fn synthesize_module(state: &mut SynthState, name: &str, arg_nets: &[NetID]) -> Result<(), CerrSpan> {
  let mut inc_nets = vec![];
  let mut inc_net_map = HashMap::<String, IncompleteNetID>::new();
  let mut net_map = HashMap::<String, NetID>::new();
  let (module, span) = state.collected_modules.get(name).unwrap();
  
  // collect module ports into incompletenet instances
  //collect_ports_to_inc_nets();
  
  // collect wire and mem decl nets into incomplete nets
  presynth_wire_mem_decls(&module.stmts, &mut inc_nets, &mut inc_net_map);
  
  // run expr presynth
  
  // resolve net colors and signals
  
  // convert all `IncompleteNet`s to real nets
  
  // run expr synth
  
  Ok(())
}

fn collect_ports_to_inc_nets(state: &mut SynthState, ports: &[PortDecl], arg_nets: &[NetID], inc_nets: &mut Vec<IncompleteNet>, inc_net_map: &mut HashMap<String, IncompleteNetID>) {
  ports.iter()
    .zip(arg_nets.iter())
    .for_each(|(port, net_id)| {
      let net = &state.netlist.nets[*net_id];
      todo!()
    })
}

fn presynth_wire_mem_decls(stmts: &[(Stmt, Span)], inc_nets: &mut Vec<IncompleteNet>, inc_net_map: &mut HashMap<String, IncompleteNetID>) {
  stmts.iter()
    .for_each(|(stmt, _)| {
      match stmt {
        Stmt::MemDecl { name, signal_class } => {
          inc_net_map.insert(name.clone(), inc_nets.len());
          inc_nets.push(IncompleteNet {
            different_color_as_net: vec![],
            different_signal_as_net: vec![],
            resolved_color: None,
            resolved_signal: None,
            ty: *signal_class,
          });
        }
        Stmt::WireDecl { name, signal_class, expr } => {
          inc_net_map.insert(name.clone(), inc_nets.len());
          inc_nets.push(IncompleteNet {
            different_color_as_net: vec![],
            different_signal_as_net: vec![],
            resolved_color: None,
            resolved_signal: None,
            ty: *signal_class,
          })
        }
        Stmt::Trigger { statements, .. } => {
          presynth_wire_mem_decls(statements, inc_nets, inc_net_map);
        }
        _ => {}
      }
    });
}

fn presynth_expr(expr: &Expr, inc_nets: &mut Vec<IncompleteNet>, inc_net_map: &HashMap<String, IncompleteNetID>, assign_result_to: IncompleteNetID) -> Result<(), CerrSpan> {
  todo!()
}

fn synth_expr(expr: &Expr, nets: &mut Vec<Net>, assign_result_to: IncompleteNetID) -> Result<(), CerrSpan> {
  todo!()
}