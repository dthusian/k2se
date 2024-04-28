use crate::err::{Cerr, CerrSpan, TypeError};
use crate::parse::ast::{Expr, Module, NetType, PortClass, Stmt, TriggerKind};
use crate::parse::span::Span;
use crate::parse::tokenizer::BinaryOp;
use crate::synth::builtins::binaryop::binary_op_to_func_name;
use crate::synth::builtins::{BuiltinFunction, FunctionArgReq};
use crate::synth::ir::{IRModule, IRModuleInst, IRStmt, IRTriggerStmt, IRValue, IRWireMemDecl};
use std::collections::HashMap;

/// Tracks program-wide validation state.
struct GlobalValidationState<'a> {
  errors: Vec<CerrSpan>,
  modules: HashMap<String, &'a Module>,
  builtins: &'a HashMap<String, Box<dyn BuiltinFunction>>,
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
  pub fn create_net(
    &mut self,
    name: &String,
    span: Span,
    ty: NetType,
    mem: bool,
    input: bool,
    port_idx: Option<usize>,
  ) {
    let prev = self.objects.insert(
      name.clone(),
      ObjectInfo {
        mem,
        input,
        exclusive_write: false,
      },
    );
    if prev.is_some() {
      self
        .global
        .errors
        .push(Cerr::MultipleDeclarations(name.clone()).with(span))
    }
    self
      .ir_objects
      .insert(name.clone(), IRWireMemDecl { ty, mem, port_idx });
  }

  /// Utility method that creates an anonymous net.
  /// All expressions must be flattened into assignments to anonymous nets.
  /// Doesn't add it to `objects` because anonymous nets cannot be
  /// referred to by code.
  pub fn create_anon_net(&mut self, ty: NetType) -> String {
    let anon = format!("$anon_{}", self.next_anon);
    self.ir_objects.insert(
      anon.clone(),
      IRWireMemDecl {
        ty,
        mem: false,
        port_idx: None,
      },
    );
    self.next_anon += 1;
    anon
  }

  /// Either returns the net itself, or an anonymous net
  /// that is connected to a trigger filter
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
        self
          .global
          .errors
          .push(Cerr::MultipleExclusiveWrites.with(span));
      }
      if info.mem && assign_type == BinaryOp::Assign && !trigger {
        self
          .global
          .errors
          .push(Cerr::MemAssignOutsideOfTrigger.with(span));
      }
      if assign_type == BinaryOp::Assign {
        self.objects.get_mut(name).unwrap().exclusive_write = true;
      }
    } else {
      self
        .global
        .errors
        .push(Cerr::NotDeclared(name.clone()).with(span));
    }
  }
}

struct ObjectInfo {
  /// Whether object is a mem
  mem: bool,
  /// Whether the object is an input wire
  input: bool,
  /// Whether the object has already been `=`-assigned to
  exclusive_write: bool,
}

/// The public method that transforms all modules.
pub fn transform_modules(
  modules: &[(Module, Span)],
  builtins: &HashMap<String, Box<dyn BuiltinFunction>>,
) -> (Vec<IRModule>, Vec<CerrSpan>) {
  let mut state = GlobalValidationState {
    errors: vec![],
    modules: Default::default(),
    builtins,
  };
  collect_modules(&mut state, modules);
  let ir_modules = modules
    .into_iter()
    .map(|v| transform_module(&mut state, v))
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
    name: module.0.name.clone(),
    ports: module.0.ports.clone(),
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
      state
        .errors
        .push(Cerr::MultipleDeclarations(prev.name.clone()).with(*span));
    }
  })
}

fn collect_module_inputs(state: &mut ModuleValidationState, module: &(Module, Span)) {
  let (module, span) = module;
  module.ports.iter().enumerate().for_each(|(i, v)| {
    state.create_net(
      &v.name,
      *span,
      v.signal_class,
      false,
      v.port_class == PortClass::In,
      Some(i),
    );
  });
}

fn collect_decls(state: &mut ModuleValidationState, stmts: &[(Stmt, Span)], in_trigger: bool) {
  stmts.into_iter().for_each(|(stmt, span)| match stmt {
    Stmt::MemDecl { name, signal_class } => {
      state.create_net(name, *span, *signal_class, true, false, None);
    }
    Stmt::WireDecl {
      name, signal_class, ..
    } => {
      state.create_net(name, *span, *signal_class, false, false, None);
    }
    Stmt::Trigger { statements, .. } => {
      if !in_trigger {
        collect_decls(state, &statements, true);
      }
    }
    _ => {}
  })
}

fn transform_stmt(
  state: &mut ModuleValidationState,
  stmt: &(Stmt, Span),
  trigger: Option<&String>,
) {
  let (stmt, span) = stmt;
  let span = *span;
  match &stmt {
    Stmt::MemDecl { .. } => {}

    Stmt::Set {
      name,
      assign_type,
      expr,
    } => {
      state.validate_set(name, *assign_type, span, trigger.is_some());
      let ir_obj = state.ir_objects.get(&name);
      if ir_obj.unwrap().mem && assign_type == &BinaryOp::Assign {
        // mem objects that are assigned to need special treatment
        // since they need to add a negated version back into it (since naive-assign actually increments it)
        
        // to fix this, generate an ad-hoc expr that subtracts the current memcell from the expr
        // unfortunately we have to clone the entire expr tree to accompish this
        let net = state.set_or_trigger(name, trigger.cloned());
        transform_expr_and_assign_to(state, &Expr::BinaryOps {
          car: Box::new(expr.clone()),
          cdr: vec![(BinaryOp::Sub, Expr::Identifier { name: name.clone() })],
        }, span, net);
      } else {
        let net = state.set_or_trigger(name, trigger.cloned());
        transform_expr_and_assign_to(state, expr, span, net);
      }
    }

    Stmt::WireDecl { name, expr, .. } => {
      if let Some(expr) = expr {
        state.validate_set(name, BinaryOp::Assign, span, trigger.is_some());
        let net = state.set_or_trigger(name, trigger.cloned());
        transform_expr_and_assign_to(state, expr, span, net);
      }
    }

    Stmt::ModuleInst {
      module: module_name,
      args,
    } => {
      // check module exists
      let module = state.global.modules.get(module_name);
      if module.is_none() {
        state
          .global
          .errors
          .push(Cerr::NotDeclared(module_name.clone()).with(span));
        return;
      }
      let module = module.unwrap();

      // check module has the same number of arguments
      if args.len() != module.ports.len() {
        state
          .global
          .errors
          .push(Cerr::WrongNumberOfModuleArgs(module.ports.len()).with(span));
      }

      let args = args
        .iter()
        .zip(module.ports.iter())
        .enumerate()
        .map(|(i, (expr, port))| {
          if let Expr::Identifier { name } = expr {
            if port.port_class == PortClass::Out {
              // validate multiple assign
              state.validate_set(name, BinaryOp::Assign, span, trigger.is_some());
            }
            Some(name.clone())
          } else {
            if port.port_class != PortClass::In {
              state
                .global
                .errors
                .push(Cerr::ExprForOutInoutPort(i).with(span));
            }
            let arg = transform_expr(state, expr, span).0;
            match arg {
              IRValue::Net(net) => Some(net),
              IRValue::Lit(v) => {
                let anon = state.create_anon_net(NetType::Single);
                state.stmts.push(IRStmt {
                  dest: anon.clone(),
                  op: "passthrough".into(),
                  args: vec![IRValue::Lit(v)],
                });
                Some(anon)
              }
              IRValue::Str(_) => None,
            }
            .map(|v| v.clone())
            .ok_or_else(|| Cerr::UnexpectedString.with(span))
            .map_err(|e| state.global.errors.push(e))
            .ok()
          }
        })
        .collect::<Option<Vec<_>>>();

      if let Some(args) = args {
        state.module_inst.push(IRModuleInst {
          name: module_name.clone(),
          args,
        });
      }
    }

    Stmt::Trigger {
      watching,
      trigger_kind,
      statements,
    } => {
      if trigger.is_some() {
        state
          .global
          .errors
          .push(Cerr::NestedTriggerBlocks.with(span));
        return;
      }
      let obj_info = state.objects.get(watching);
      if obj_info.is_none() {
        state
          .global
          .errors
          .push(Cerr::NotDeclared(watching.clone()).with(span));
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
          TriggerKind::Raw => unreachable!(),
        };
        state.stmts.push(IRStmt {
          dest: anon.clone(),
          op: trigger_fn.to_owned(),
          args: vec![IRValue::Net(watching.clone())],
        });
        anon
      };
      for inner in statements {
        transform_stmt(state, inner, Some(&trigger));
      }
    }
  }
}

/// Returns a tuple of (result net, is anonymous net?, net type)
/// The expectation is that if the net was anonymous, then it is safe to change
/// the stmt that assigns it to a different net.
fn transform_expr(state: &mut ModuleValidationState, expr: &Expr, span: Span) -> (IRValue, bool) {
  match expr {
    Expr::Identifier { name } => {
      if !state.objects.contains_key(name) {
        state
          .global
          .errors
          .push(Cerr::NotDeclared(name.clone()).with(span));
      }
      (IRValue::Net(name.clone()), false)
    }

    Expr::BinaryOps { car, cdr } => {
      let mut resolved_arg = Some(transform_expr(state, car, span).0);
      cdr.iter().for_each(|(op, expr)| {
        let (irv, _) = transform_expr(state, expr, span);
        resolved_arg = Some(transform_single_binary_op(
          state,
          *op,
          resolved_arg.take().unwrap(),
          irv,
          span,
        ));
      });
      (resolved_arg.unwrap(), true)
    }

    Expr::FnCall { args, func } => {
      let args = args
        .iter()
        .map(|expr| transform_expr(state, expr, span).0)
        .collect::<Vec<_>>();
      if let Some(func_box) = state.global.builtins.get(func) {
        if args.len() != func_box.arg_ty().len() {
          state
            .global
            .errors
            .push(Cerr::WrongNumberOfFunctionArgs(func_box.arg_ty().len()).with(span))
        }
        func_box
          .arg_ty()
          .iter()
          .zip(args.iter())
          .enumerate()
          .for_each(|(i, (a1, a2))| {
            // do type checking
            match a1 {
              FunctionArgReq::Any => {}
              FunctionArgReq::Net(ty) => {
                match a2 {
                  IRValue::Net(net) => {
                    // unwrap: IRValue nets should always be validated
                    let ty2 = state.ir_objects.get(net).unwrap().ty;
                    if &ty2 != ty {
                      state.global.errors.push(
                        Cerr::TypeErrArgMismatch(
                          i,
                          func.clone(),
                          TypeError {
                            src_ty: format!("{:?}", ty2),
                            dst_ty: format!("{:?}", ty),
                          },
                        )
                        .with(span),
                      );
                    }
                  }
                  IRValue::Lit(_) => {
                    state.global.errors.push(
                      Cerr::TypeErrArgMismatch(
                        i,
                        func.clone(),
                        TypeError {
                          src_ty: "Literal".into(),
                          dst_ty: format!("{:?}", ty),
                        },
                      )
                      .with(span),
                    );
                  }
                  IRValue::Str(_) => {
                    state.global.errors.push(Cerr::UnexpectedString.with(span));
                  }
                }
              }
              FunctionArgReq::SingleOrLit => match a2 {
                IRValue::Net(net) => {
                  let ty2 = state.ir_objects.get(net).unwrap().ty;
                  if ty2 != NetType::Single {
                    state.global.errors.push(
                      Cerr::TypeErrArgMismatch(
                        i,
                        func.clone(),
                        TypeError {
                          src_ty: format!("{:?}", ty2),
                          dst_ty: "Single".into(),
                        },
                      )
                      .with(span),
                    );
                  }
                }
                IRValue::Lit(_) => {}
                IRValue::Str(_) => {
                  state.global.errors.push(Cerr::UnexpectedString.with(span));
                }
              },
              FunctionArgReq::String => {
                if !matches!(a2, &IRValue::Str(_)) {
                  state
                    .global
                    .errors
                    .push(Cerr::ExpectedString(i, func.clone()).with(span))
                }
              }
            }
          });
        // create anon net
        let anon = state.create_anon_net(func_box.return_ty());
        state.stmts.push(IRStmt {
          dest: anon.clone(),
          op: func.clone(),
          args,
        });
        (IRValue::Net(anon), true)
      } else {
        state
          .global
          .errors
          .push(Cerr::UnknownFunction(func.clone()).with(span));
        (IRValue::Net("$error".into()), false)
      }
    }

    Expr::Literal { val } => (IRValue::Lit(*val), false),

    Expr::StringLiteral { str } => (IRValue::Str(str.clone()), false),
  }
}

fn transform_expr_and_assign_to(
  state: &mut ModuleValidationState,
  expr: &Expr,
  span: Span,
  assign_to: String,
) {
  let (result, is_anon) = transform_expr(state, expr, span);
  let ty = state.ir_objects[&assign_to].ty;
  match get_type(state, &result) {
    Ok(ty2) => {
      if ty != ty2 {
        state.global.errors.push(
          Cerr::TypeErrorGeneric(TypeError {
            src_ty: format!("{:?}", ty2),
            dst_ty: format!("{:?}", ty),
          })
          .with(span),
        )
      }
    }
    Err(err) => state.global.errors.push(err.with(span)),
  }
  if !is_anon {
    state.stmts.push(IRStmt {
      dest: assign_to,
      op: "passthrough".into(),
      args: vec![result],
    });
  } else {
    // unwrap: if is_anon == true, it must be an anonymous net (which is a net)
    let result = result.into_net().unwrap();
    for v in state.stmts.iter_mut().rev() {
      // find the statement that assigns to the net
      if v.dest == result {
        v.dest = assign_to;
        break;
      }
    }
  }
}

fn transform_single_binary_op(
  state: &mut ModuleValidationState,
  op: BinaryOp,
  a: IRValue,
  b: IRValue,
  span: Span,
) -> IRValue {
  let ty1 = get_type(state, &a)
    .map_err(|v| {
      state.global.errors.push(v.with(span));
    })
    .unwrap_or(NetType::Single);
  let ty2 = get_type(state, &b)
    .map_err(|v| {
      state.global.errors.push(v.with(span));
    })
    .unwrap_or(NetType::Single);
  if ty1 == NetType::Mixed && ty2 == NetType::Mixed && op != BinaryOp::Add && op != BinaryOp::Sub {
    state
      .global
      .errors
      .push(Cerr::InvalidOpOnMixedNets(op).with(span))
  }
  let res_ty = if ty1 == NetType::Mixed || ty2 == NetType::Mixed {
    NetType::Mixed
  } else {
    NetType::Single
  };
  let func = binary_op_to_func_name(op).to_owned();
  let anon = state.create_anon_net(res_ty);
  state.stmts.push(IRStmt {
    dest: anon.clone(),
    op: func,
    args: vec![a, b],
  });
  IRValue::Net(anon)
}

fn get_type(state: &mut ModuleValidationState, irv: &IRValue) -> Result<NetType, Cerr> {
  Ok(match irv {
    IRValue::Net(name) => state
      .ir_objects
      .get(name)
      .map(|v| v.ty)
      .unwrap_or(NetType::Single),
    IRValue::Lit(_) => NetType::Single,
    _ => return Err(Cerr::UnexpectedString),
  })
}
