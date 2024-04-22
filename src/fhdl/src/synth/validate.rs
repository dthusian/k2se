use std::collections::HashMap;
use crate::err::{Cerr, CerrSpan};
use crate::parse::ast::{Expr, Module, NetType, PortClass, Stmt, TriggerKind};
use crate::parse::span::Span;
use crate::parse::tokenizer::BinaryOp;
use crate::synth::ir::{IRModule, IRModuleInst, IRNetOrLiteral, IRStmt, IRTriggerStmt, IRWireMemDecl};

/// Tracks program-wide validation state.
struct GlobalValidationState<'a> {
  errors: Vec<CerrSpan>,
  modules: HashMap<String, &'a Module>
}

/// Tracks module-wide validation state.
struct ModuleValidationState<'a, 'b> {
  global: &'b mut GlobalValidationState<'a>,
  /// `ir_objects` is what ends up in the transformed IR, while
  /// `objects` is just for validation
  ir_objects: HashMap<String, IRWireMemDecl>,
  objects: HashMap<String, ObjectInfo>,
  stmts: Vec<IRStmt>,
  trigger_stmts: Vec<IRTriggerStmt>,
  module_inst: Vec<IRModuleInst>,
  next_anon: u64,
}

impl<'a, 'b> ModuleValidationState<'a, 'b> {
  /// Utility method to just create a net.
  /// Adds it to both `ir_objects` and `objects`.
  pub fn create_net(&mut self, name: &String, span: Span, ty: NetType, mem: bool, input: bool, port_idx: Option<usize>) {
    let prev = self.objects.insert(name.clone(), ObjectInfo {
      mem,
      input,
      exclusive_write: false,
    });
    if prev.is_some() {
      self.global.errors.push(Cerr::MultipleDeclarations(name.clone()).with(span))
    }
    self.ir_objects.insert(name.clone(), IRWireMemDecl {
      ty,
      mem,
      port_idx,
    });
  }
  
  /// Utility method that creates an anonymous net.
  /// All expressions must be flattened into assignments to anonymous nets.
  /// Doesn't add it to `objects` because anonymous nets cannot be
  /// referred to by code.
  pub fn create_anon_net(&mut self, ty: NetType) -> String {
    let anon = format!("$anon_{}", self.next_anon);
    self.ir_objects.insert(anon.clone(), IRWireMemDecl {
      ty,
      mem: false,
      port_idx: None,
    });
    anon
  }
  
  /// Either returns the net itself, or an anonymous net
  /// that is connected to  
  pub fn set_or_trigger(&mut self, name: &String, on_trigger: Option<String>) -> String {
    if let Some(on_trigger) = on_trigger {
      let decl = self.ir_objects.get(name).unwrap();
      let anon = self.create_anon_net(decl.ty);
      self.trigger_stmts.push(IRTriggerStmt {
        dest: name.clone(),
        src: anon.clone(),
        on: on_trigger,
      });
      anon
    } else {
      name.clone()
    }
  }
  
  /// Validates a set statement or wire decl with expression.
  pub fn validate_set(&mut self, name: &String, assign_type: BinaryOp, span: Span, trigger: bool) {
    let info = self.objects.get(name);
    if let Some(info) = info {
      if info.input {
        self.global.errors.push(Cerr::WriteToInput.with(span));
      }
      if info.exclusive_write {
        self.global.errors.push(Cerr::MultipleExclusiveWrites.with(span));
      }
      if info.mem && assign_type == BinaryOp::Assign && !trigger {
        self.global.errors.push(Cerr::MemAssignOutsideOfTrigger.with(span));
      }
      if assign_type == BinaryOp::Assign {
        self.objects.get_mut(name).unwrap().exclusive_write = true;
      }
    } else {
      self.global.errors.push(Cerr::NotDeclared(name.clone()).with(span));
    }
  }
}

struct ObjectInfo {
  /// Whether object is a mem
  mem: bool,
  /// Whether the object is an input wire
  input: bool,
  /// Whether the object has already been `=`-assigned to
  exclusive_write: bool
}

/// The public method that transforms all modules.
pub fn transform_modules(modules: &[(Module, Span)]) -> (Vec<IRModule>, Vec<CerrSpan>) {
  let mut state = GlobalValidationState {
    errors: vec![],
    modules: Default::default(),
  };
  collect_modules(&mut state, modules);
  let ir_modules = modules
    .into_iter()
    .map(|v| {
      transform_module(&mut state, v)
    })
    .collect::<Vec<_>>();
  (ir_modules, state.errors)
}

fn transform_module(state: &mut GlobalValidationState, module: &(Module, Span)) -> IRModule {
  let mut state = ModuleValidationState {
    global: state,
    ir_objects: Default::default(),
    objects: Default::default(),
    stmts: vec![],
    trigger_stmts: vec![],
    module_inst: vec![],
    next_anon: 0,
  };
  collect_module_inputs(&mut state, module);
  collect_decls(&mut state, &module.0.stmts, false);
  module.0.stmts.iter().for_each(|v| {
    transform_stmt(&mut state, v, None);
  });
  IRModule {
    objects: state.ir_objects,
    stmts: state.stmts,
    trigger_stmt: state.trigger_stmts,
    module_inst: state.module_inst,
  }
}

fn collect_modules<'a>(state: &'_ mut GlobalValidationState<'a>, modules: &'a [(Module, Span)]) {
  modules.into_iter().for_each(|(module, span)| {
    let prev = state.modules.insert(module.name.clone(), module);
    if let Some(prev) = prev {
      state.errors.push(Cerr::MultipleDeclarations(prev.name.clone()).with(*span));
    }
  })
}

fn collect_module_inputs(state: &mut ModuleValidationState, module: &(Module, Span)) {
  let (module, span) = module;
  module.ports
    .iter()
    .enumerate()
    .for_each(|(i, v)| {
      state.create_net(&v.name, *span, v.signal_class, false, v.port_class == PortClass::In, Some(i));
    });
}

fn collect_decls(state: &mut ModuleValidationState, stmts: &[(Stmt, Span)], in_trigger: bool) {
  stmts.into_iter().for_each(|(stmt, span)| {
    match stmt {
      Stmt::MemDecl { name, signal_class } => {
        state.create_net(name, *span, *signal_class, true, false, None);
      }
      Stmt::WireDecl { name, signal_class, .. } => {
        state.create_net(name, *span, *signal_class, false, false, None);
      }
      Stmt::Trigger { statements, .. } => {
        if !in_trigger {
          collect_decls(state, &statements, true);
        }
      }
      _ => {}
    }
  })
}

fn transform_stmt(state: &mut ModuleValidationState, stmt: &(Stmt, Span), trigger: Option<&String>) {
  let (stmt, span) = stmt;
  let span = *span;
  match &stmt {
    Stmt::MemDecl { .. } => { }
    
    Stmt::Set { name, assign_type, expr } => {
      state.validate_set(name, *assign_type, span, trigger.is_some());
      let net = state.set_or_trigger(name, trigger.cloned());
      transform_expr(state, expr, span, net);
    }
    
    Stmt::WireDecl { name, expr, .. } => {
      if let Some(expr) = expr {
        state.validate_set(name, BinaryOp::Assign, span, trigger.is_some());
        let net = state.set_or_trigger(name, trigger.cloned());
        transform_expr(state, expr, span, net);
      }
    }
    
    Stmt::ModuleInst { module: module_name, args } => {
      // validate
      let module = state.global.modules.get(module_name);
      if let Some(module) = module {
        if args.len() != module.ports.len() {
          state.global.errors.push(Cerr::WrongNumberOfModuleArgs(module.ports.len()).with(span));
        }
        let args = args.iter()
          .zip(module.ports.iter())
          .enumerate()
          .map(|(i, (expr, port))| {
            let is_ident = matches!(expr, &Expr::Identifier { .. });
            if port.port_class != PortClass::In && !is_ident {
              state.global.errors.push(Cerr::ExprForOutInoutPort(i).with(span));
            }
            // transform
            if let Expr::Identifier { name } = expr {
              name.clone()
            } else {
              let anon = state.create_anon_net(port.signal_class);
              transform_expr(state, expr, span, anon.clone());
              anon
            }
          })
          .collect::<Vec<_>>();
        // transform
        state.module_inst.push(IRModuleInst {
          name: module_name.clone(),
          args,
        });
      } else {
        state.global.errors.push(Cerr::NotDeclared(module_name.clone()).with(span));
      }
    }
    
    Stmt::Trigger { watching, trigger_kind, statements , } => {
      if trigger.is_some() {
        state.global.errors.push(Cerr::NestedTriggerBlocks.with(span));
        return;
      }
      let obj_info = state.objects.get(watching);
      if obj_info.is_none() {
        state.global.errors.push(Cerr::NotDeclared(watching.clone()).with(span));
      }
      // if it's raw mode, use the trigger signal directly
      // or generate a trigger detector
      let trigger = if trigger_kind == &TriggerKind::Raw {
        watching.clone()
      } else {
        let anon = state.create_anon_net(NetType::Single);
        let trigger_fn = match trigger_kind {
          TriggerKind::Increasing => "trig_inc",
          TriggerKind::Decreasing => "trig_dec",
          TriggerKind::Changed => "trig_chg",
          TriggerKind::Raw => unreachable!()
        };
        state.stmts.push(IRStmt {
          dest: anon.clone(),
          op: trigger_fn.to_owned(),
          args: vec![IRNetOrLiteral::Net(watching.clone())],
        });
        anon
      };
      for inner in statements {
        transform_stmt(state, inner, Some(&trigger));
      }
    }
    
  }
}

fn transform_expr(state: &mut ModuleValidationState, expr: &Expr, span: Span, assign_to: String) -> NetType {
  match expr {
    Expr::Identifier { name } => {
      if !state.objects.contains_key(name) {
        state.global.errors.push(Cerr::NotDeclared(name.clone()).with(span));
      }
      todo!()
    }
    Expr::BinaryOps { car, cdr } => {
      todo!()
    }
    Expr::FnCall { args, .. } => {
      todo!()
    }
    Expr::Literal { .. } => {
      todo!()
    }
  }
}

fn expr_ident_or_literal(expr: &Expr) -> Option<IRNetOrLiteral> {
  match expr {
    Expr::Identifier { name } => Some(IRNetOrLiteral::Net(name.clone())),
    Expr::Literal { val } => Some(IRNetOrLiteral::Lit(*val)),
    _ => None
  }
}