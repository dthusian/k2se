//! AST parsing. All structs here are part of the AST, and
//! have a parse function that returns `Self` or `(Self, Span)`. 

use crate::err::{Cerr, CerrSpan};
use crate::parse::span::{Span};
use crate::parse::tokenizer::{BinaryOp, Token};
use crate::parse::tokenstream::Cursor;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Program {
  pub version: Version,
  pub modules: Vec<(Module, Span)>
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
  pub name: String,
  pub ports: Vec<PortDecl>,
  pub stmts: Vec<(Stmt, Span)>
}

impl Module {
  pub fn parse(tokens: &Cursor) -> Result<(Self, Span), CerrSpan> {
    let start_span = tokens.next_assert(&Token::Name("module".into()))?;
    let (name, _) = tokens.next_map(
      |v| v
        .get_name()
        .map(|v| v.to_owned())
        .ok_or(Cerr::UnexpectedTokenType("identifier"))
    )?;
    let ports = parse_list_paren_comma(tokens, PortDecl::parse)?;
    let stmts = parse_list_brace_semi(tokens, Stmt::parse)?;
    tokens.rewind(1);
    let end_span = tokens.next()?.1;
    Ok((Module {
      name,
      ports,
      stmts,
    }, start_span.union(end_span)))
  }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PortDecl {
  pub port_class: PortClass,
  pub signal_class: NetType,
  pub name: String,
}

impl PortDecl {
  pub fn parse(tokens: &Cursor) -> Result<Self, CerrSpan> {
    Ok(PortDecl {
      port_class: PortClass::parse(tokens)?.0,
      signal_class: NetType::parse(tokens)?.0,
      name: tokens.next_identifier()?.0,
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
      Ok(match v {
        Token::Name(name) if name == "in" => PortClass::In,
        Token::Name(name) if name == "out" => PortClass::Out,
        Token::Name(name) if name == "inout" => PortClass::InOut,
        _ => return Err(Cerr::UnexpectedToken(vec!["in".into(), "out".into(), "inout".into()]))
      })
    })
  }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Stmt {
  MemDecl {
    name: String,
  },
  Set {
    name: String,
    assign_type: BinaryOp,
    expr: Expr,
  },
  WireDecl {
    name: String,
    expr: Option<Expr>,
  },
  ModuleInst {
    module: String,
    args: Vec<Expr>,
  },
  Trigger {
    watching: String,
    trigger_kind: TriggerKind,
    statements: Vec<(Stmt, Span)>
  }
}

impl Stmt {
  pub fn parse(tokens: &Cursor) -> Result<(Self, Span), CerrSpan> {
    let (kw, start) = tokens.next()?;
    Ok(match kw {
      
      Token::Name(kw) if kw == "mem" => {
        let name = tokens.next_identifier()?.0;
        let end = tokens.peek_assert(&Token::Semicolon)?;
        (Stmt::MemDecl { name }, start.union(end))
      },
      
      Token::Name(kw) if kw == "set" => {
        let name = tokens.next_identifier()?.0;
        let assign_type = tokens.next_map(|v|
          v.get_op()
            .ok_or(Cerr::UnexpectedToken(vec!["=".into(), "+=".into()]))
          )?.0;
        let expr = Expr::parse(tokens)?;
        let end = tokens.peek_assert(&Token::Semicolon)?;
        (Stmt::Set {
          name,
          assign_type,
          expr,
        }, start.union(end))
      },
      
      Token::Name(kw) if kw == "wire" => {
        let name = tokens.next_identifier()?.0;
        let (maybe_assign, _) = tokens.peek()?;
        if maybe_assign != &Token::Semicolon {
          tokens.next_assert(&Token::Op(BinaryOp::Assign))?;
          let expr = Expr::parse(tokens)?;
          let end = tokens.peek_assert(&Token::Semicolon)?;
          (Stmt::WireDecl {
            name,
            expr: Some(expr),
          }, start.union(end))
        } else {
          let end = tokens.peek_assert(&Token::Semicolon)?;
          (Stmt::WireDecl {
            name,
            expr: None,
          }, start.union(end))
        }
      },
      
      Token::Name(kw) if kw == "inst" => {
        let name = tokens.next_identifier()?.0;
        let args = parse_list_paren_comma(tokens, Expr::parse)?;
        let end = tokens.peek_assert(&Token::Semicolon)?;
        (Stmt::ModuleInst {
          module: name,
          args,
        }, start.union(end))
      },
      
      Token::Name(kw) if kw == "trigger" => {
        let name = tokens.next_identifier()?.0;
        let trigger_kind = TriggerKind::parse(tokens)?.0;
        let stmts = parse_list_brace_semi(tokens, Stmt::parse)?;
        let end = tokens.peek_assert(&Token::Semicolon)?;
        (Stmt::Trigger {
          watching: name,
          trigger_kind,
          statements: stmts,
        }, start.union(end))
      }
      
      _ => return Err(Cerr::UnexpectedToken(vec!["mem".into(), "set".into(), "wire".into(), "inst".into(), "trigger".into()]).with(start))
    })
  }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TriggerKind {
  Increasing,
  Decreasing,
  Changed,
  Raw,
}

impl TriggerKind {
  pub fn parse(tokens: &Cursor) -> Result<(Self, Span), CerrSpan> {
    tokens.next_map(|v| {
      Ok(match v {
        Token::Name(name) if name == "increasing" => TriggerKind::Increasing,
        Token::Name(name) if name == "decreasing" => TriggerKind::Decreasing,
        Token::Name(name) if name == "changed" => TriggerKind::Changed,
        Token::Name(name) if name == "raw" => TriggerKind::Raw,
        _ => return Err(Cerr::UnexpectedToken(vec!["increasing".into(), "decreasing".into(), "changed".into(), "raw".into()]))
      })
    })
  }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum NetType {
  Single,
  Mixed
}

impl NetType {
  pub fn parse(tokens: &Cursor) -> Result<(Self, Span), CerrSpan> {
    tokens.next_map(|v| {
      Ok(match v {
        Token::Name(name) if name == "single" => NetType::Single,
        Token::Name(name) if name == "mixed" => NetType::Mixed,
        _ => return Err(Cerr::UnexpectedToken(vec!["in".into(), "out".into(), "inout".into()]))
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
}

impl Expr {
  pub fn parse(tokens: &Cursor) -> Result<Self, CerrSpan> {
    Self::parse_with_prec(tokens, 6)
  }

  pub fn parse_with_prec(tokens: &Cursor, prec: u32) -> Result<Self, CerrSpan> {
    Ok(if prec == 0 {
      let (t, span) = tokens.next()?;
      match t {
        Token::Name(name) => {
          // disambiguate between function call and variable use
          if matches!(tokens.peek_or_eof(), Some((&Token::LParen, _))) {
            // function call
            let args = parse_list_paren_comma(tokens, Expr::parse)?;
            Expr::FnCall {
              func: name.clone(),
              args,
            }
          } else {
            // peeked token is binary op or semicolon or something, anyways end of expression
            // therefore, this is a variable
            Expr::Identifier { name: name.clone() }
          }
        }
        Token::Literal(i) => {
          // literal expression
          Expr::Literal { val: *i }
        }
        Token::LParen => {
          // braced expression
          let expr = Expr::parse(tokens)?;
          tokens.next_assert(&Token::RParen)?;
          expr
        }
        _ => return Err(Cerr::InvalidExpr.with(span))
      }
    } else {
      let car = Expr::parse_with_prec(tokens, prec - 1)?;
      let mut cdr = vec![];
      while matches!(tokens.peek_or_eof(), Some((&Token::Op(op), _)) if op.precedence() == prec) {
        // unwrap: can only be op if the above matches! passes
        let op = tokens.next()?.0.get_op().unwrap();
        let expr = Expr::parse_with_prec(tokens, prec - 1)?;
        cdr.push((op, expr));
      }
      if cdr.is_empty() {
        car
      } else {
        Expr::BinaryOps {
          car: Box::new(car),
          cdr,
        }
      }
    })
  }
}


fn parse_list_paren_comma<T>(tokens: &Cursor, f: impl FnMut(&Cursor) -> Result<T, CerrSpan>) -> Result<Vec<T>, CerrSpan> {
  parse_list(tokens, f, &Token::LParen, &Token::RParen, &Token::Comma)
}

fn parse_list_brace_semi<T>(tokens: &Cursor, f: impl FnMut(&Cursor) -> Result<T, CerrSpan>) -> Result<Vec<T>, CerrSpan> {
  parse_list(tokens, f, &Token::LBrace, &Token::RBrace, &Token::Semicolon)
}

fn parse_list<T>(tokens: &Cursor, mut f: impl FnMut(&Cursor) -> Result<T, CerrSpan>, start: &Token, end: &Token, delim: &Token) -> Result<Vec<T>, CerrSpan> {
  let mut elems = vec![];
  tokens.next_assert(start)?;
  if tokens.peek()?.0 == end {
    tokens.next()?;
    return Ok(elems);
  }
  loop {
    elems.push(f(tokens)?);
    let (token, span ) = tokens.next()?;
    if token == end { break }
    if token == delim {
      // to handle trailing commas
      let peek = tokens.peek()?.0;
      if peek == end {
        tokens.next()?;
        break
      }
      continue
    }
    return Err(Cerr::UnexpectedToken(vec![end.to_string(), delim.to_string()]).with(span));
  }
  Ok(elems)
}