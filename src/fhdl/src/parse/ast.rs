use crate::err::{Cerr, CerrSpan};
use crate::parse::span::{Span};
use crate::parse::tokenizer::{BinaryOp, Token};
use crate::parse::tokenstream::Cursor;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Program {
  pub version: Version,
  pub modules: Vec<Module>
}

impl Program {
  pub fn parse(tokens: &Cursor) -> Result<Self, CerrSpan> {
    let version = Version::parse(tokens)?;
    let mut modules = vec![];
    loop {
      let peeker = tokens.clone();
      let Some((token, span)) = peeker.next_or_eof() else { break; };
      match &token {
        Token::Name(name) if name == "module" => {
          modules.push(Module::parse(tokens)?);
        }
        _ => return Err(Cerr::UnexpectedToken(vec!["module".into()]).with(span))
      }
    }
    Ok(Program {
      version,
      modules,
    })
  }
}


#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Version {
  V2
}

impl Version {
  pub fn parse(tokens: &Cursor) -> Result<Self, CerrSpan> {
    tokens.next_assert(&Token::Name("version".into()))?;
    let (ver, span) = tokens.next()?;
    match ver {
      Token::Literal(2) => Version::V2,
      _ => return Err(Cerr::UnexpectedToken(vec!["2".into()]).with(span))
    };
    Ok(Version::V2)
  }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Module {
  pub span: Span,
  pub name: String,
  pub ports: Vec<PortDecl>,
  pub stmts: Vec<Stmt>
}

impl Module {
  pub fn parse(tokens: &Cursor) -> Result<Self, CerrSpan> {
    let start_span = tokens.next_assert(&Token::Name("version".into()))?;
    let (name, _) = tokens.next_map(
      |v| v
        .get_name()
        .map(|v| v.to_owned())
        .ok_or(Cerr::UnexpectedTokenType("identifier"))
    )?;
    tokens.next_assert(&Token::LParen)?;
    let mut ports = vec![];
    loop {
      ports.push(PortDecl::parse(tokens)?);
      let (token, span) = tokens.next()?;
      match token {
        Token::RParen => break,
        Token::Comma => continue,
        _ => return Err(Cerr::UnexpectedToken(vec![",".into(), ")".into()]).with(span))
      }
    }
    Ok(Module {
      span: start_span.union(todo!()),
      name,
      ports: vec![],
      stmts: vec![],
    })
  }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PortDecl {
  pub port_class: PortClass,
  pub signal_class: SignalClass,
  pub name: String,
}

impl PortDecl {
  pub fn parse(tokens: &Cursor) -> Result<Self, CerrSpan> {
    Ok(PortDecl {
      port_class: PortClass::parse(tokens)?.0,
      signal_class: SignalClass::Single,
      name: "".to_string(),
    })
  }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum PortClass {
  In, Out, InOut,
}

impl PortClass {
  pub fn parse(tokens: &Cursor) -> Result<(Self, Span), CerrSpan> {
    tokens.next_map(|v| {
      let cerr = Cerr::UnexpectedToken(vec!["in".into(), "out".into(), "inout".into()]);
      Ok(match v {
        Token::Name(name) => match name.as_str() { 
          "in" => PortClass::In,
          "out" => PortClass::Out,
          "inout" => PortClass::InOut,
          _ => return Err(cerr),
        },
        _ => return Err(cerr)
      })
    })
  }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Stmt {
  MemDecl {
    name: String,
  },
  MemSet {
    name: String,
    add_assign: bool,
    expr: Expr,
  },
  WireDecl {
    name: String,
    expr: Expr,
  },
  ModuleInst {
    module: String,
    args: Vec<Expr>,
  },
  Trigger {
    wire: String,
    trigger_kind: TriggerKind,
    statements: Vec<Stmt>
  }
}

impl Stmt {
  
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TriggerKind {
  Increasing,
  Decreasing,
  Changed,
  Raw,
}

impl TriggerKind {
  
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SignalClass {
  Single,
  Mixed
}

impl SignalClass {
  pub fn parse(tokens: &Cursor) -> Result<(Self, Span), CerrSpan> {
    tokens.next_map(|v| {
      let cerr = Cerr::UnexpectedToken(vec!["in".into(), "out".into(), "inout".into()]);
      Ok(match v {
        Token::Name(name) => match name.as_str() {
          "single" => SignalClass::Single,
          "mixed" => SignalClass::Mixed,
          _ => return Err(cerr),
        },
        _ => return Err(cerr)
      })
    })
  }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Expr {
  Identifier {
    name: String,
  },
  Literal {
    val: i32,
  },
  FnCall {
    func: String,
    args: Vec<Expr>,
  },
  BinaryOps {
    car: Box<Expr>,
    cdr: Vec<(BinaryOp, Expr)>
  },
  Braced {
    inner: Box<Expr>
  },
}