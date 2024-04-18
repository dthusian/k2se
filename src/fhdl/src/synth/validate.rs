use std::collections::HashMap;
use crate::err::{Cerr, CerrSpan};
use crate::parse::ast::{Expr, Module, PortClass, Stmt};
use crate::parse::span::Span;
use crate::parse::tokenizer::BinaryOp;

/// Tracks program-wide validation state.
struct GlobalValidationState<'a> {
  errors: Vec<CerrSpan>,
  modules: HashMap<String, &'a Module>
}

/// Tracks module-wide validation state.
struct ModuleValidationState<'a, 'b> {
  global: &'b mut GlobalValidationState<'a>,
  objects: HashMap<String, ObjectInfo>
}

struct ObjectInfo {
  /// Whether object is a mem
  mem: bool,
  /// Whether the object is an input wire
  input: bool,
  /// Whether the object has already been `=`-assigned to
  exclusive_write: bool
}

pub fn validate_modules(modules: &[(Module, Span)]) -> Vec<CerrSpan> {
  let mut state = GlobalValidationState {
    errors: vec![],
    modules: Default::default(),
  };
  collect_modules(&mut state, modules);
  modules
    .into_iter()
    .for_each(|v| {
      validate_module(&mut state, v);
    });
  state.errors
}

fn validate_module(state: &mut GlobalValidationState, module: &(Module, Span)) {
  let mut state = ModuleValidationState {
    global: state,
    objects: Default::default(),
  };
  collect_module_inputs(&mut state, module);
  collect_decls(&mut state, &module.0.stmts, false);
  module.0.stmts.iter().for_each(|v| {
    validate_stmt(&mut state, v, false);
  })
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
  module.0.ports
    .iter()
    .for_each(|v| {
      let prev = state.objects.insert(v.name.clone(), ObjectInfo {
        mem: false,
        input: v.port_class == PortClass::In,
        exclusive_write: false,
      });
      if prev.is_some() {
        state.global.errors.push(Cerr::MultipleDeclarations(v.name.clone()).with(module.1))
      }
    });
}

fn collect_decls(state: &mut ModuleValidationState, stmts: &[(Stmt, Span)], in_trigger: bool) {
  stmts.into_iter().for_each(|(stmt, span)| {
    match stmt {
      Stmt::MemDecl { name } => {
        let prev = state.objects.insert(name.clone(), ObjectInfo {
          mem: true,
          input: false,
          exclusive_write: false,
        });
        if prev.is_some() {
          state.global.errors.push(Cerr::MultipleDeclarations(name.clone()).with(*span))
        }
      }
      Stmt::WireDecl { name, .. } => {
        let prev = state.objects.insert(name.clone(), ObjectInfo {
          mem: false,
          input: false,
          exclusive_write: false,
        });
        if prev.is_some() {
          state.global.errors.push(Cerr::MultipleDeclarations(name.clone()).with(*span))
        }
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

fn validate_stmt(state: &mut ModuleValidationState, stmt: &(Stmt, Span), in_trigger: bool) {
  let (stmt, span) = stmt;
  let span = *span;
  match &stmt {
    Stmt::MemDecl { .. } => { }
    Stmt::Set { name, assign_type, expr } => {
      let info = state.objects.get(name);
      if let Some(info) = info {
        if info.input {
          state.global.errors.push(Cerr::WriteToInput.with(span));
        }
        if info.exclusive_write {
          state.global.errors.push(Cerr::MultipleExclusiveWrites.with(span));
        }
        if info.mem && assign_type == &BinaryOp::Assign && !in_trigger {
          state.global.errors.push(Cerr::MemAssignOutsideOfTrigger.with(span));
        }
        if assign_type == &BinaryOp::Assign {
          state.objects.get_mut(name).unwrap().exclusive_write = true;
        }
      } else {
        state.global.errors.push(Cerr::NotDeclared(name.clone()).with(span));
      }
      validate_expr(state, expr, span);
    }
    Stmt::WireDecl { name, expr } => {
      if let Some(expr) = expr {
        state.objects.get_mut(name).unwrap().exclusive_write = true;
        validate_expr(state, expr, span);
      }
    }
    Stmt::ModuleInst { module: module_name, args } => {
      let module = state.global.modules.get(module_name);
      if let Some(module) = module {
        if args.len() != module.ports.len() {
          state.global.errors.push(Cerr::WrongNumberOfModuleArgs(module.ports.len()).with(span));
        }
        args.iter().zip(module.ports.iter()).enumerate().for_each(|(i, (expr, port))| {
          if port.port_class != PortClass::In {
            let is_ident = matches!(expr, &Expr::Identifier { .. });
            if !is_ident {
              state.global.errors.push(Cerr::ExprForOutInoutPort(i).with(span));
            }
          }
        })
      } else {
        state.global.errors.push(Cerr::NotDeclared(module_name.clone()).with(span));
      }
    }
    Stmt::Trigger { watching, statements , .. } => {
      if in_trigger {
        state.global.errors.push(Cerr::NestedTriggerBlocks.with(span));
      }
      let obj_info = state.objects.get(watching);
      if obj_info.is_none() {
        state.global.errors.push(Cerr::NotDeclared(watching.clone()).with(span));
      }
      for inner in statements {
        validate_stmt(state, inner, true);
      }
    }
  }
}

fn validate_expr(state: &mut ModuleValidationState, expr: &Expr, span: Span) {
  match expr {
    Expr::Identifier { name } => {
      if !state.objects.contains_key(name) {
        state.global.errors.push(Cerr::NotDeclared(name.clone()).with(span));
      }
    }
    Expr::BinaryOps { car, cdr } => {
      validate_expr(state, car, span);
      cdr.iter().for_each(|(_, expr)| {
        validate_expr(state, expr, span);
      });
    }
    Expr::FnCall { args, .. } => {
      args.iter().for_each(|v| {
        validate_expr(state, v, span);
      })
    }
    _ => {}
  }
}