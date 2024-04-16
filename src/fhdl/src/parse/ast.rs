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
  pub stmts: Vec<(Stmt, Span)>
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
    let ports = parse_list_paren_comma(tokens, PortDecl::parse)?;
    let stmts = parse_list_brace_semi(tokens, Stmt::parse)?;
    Ok(Module {
      span: start_span.union(todo!()),
      name,
      ports,
      stmts,
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
      signal_class: SignalClass::parse(tokens)?.0,
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
  MemSet {
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
        (Stmt::MemDecl { name }, end)
      },
      
      Token::Name(kw) if kw == "set" => {
        let name = tokens.next_identifier()?.0;
        let assign_type = tokens.next_map(|v|
          v.get_op()
            .ok_or(Cerr::UnexpectedToken(vec!["=".into(), "+=".into()]))
          )?.0;
        let expr = Expr::parse(tokens)?;
        let end = tokens.peek_assert(&Token::Semicolon)?;
        (Stmt::MemSet {
          name,
          assign_type,
          expr,
        }, end)
      },
      
      Token::Name(kw) if kw == "wire" => {
        let name = tokens.next_identifier()?.0;
        let (maybe_assign, maybe_end) = tokens.next()?;
        if maybe_assign != &Token::Semicolon {
          tokens.next_assert(&Token::Op(BinaryOp::Assign))?;
          let expr = Expr::parse(tokens)?;
          let end = tokens.peek_assert(&Token::Semicolon)?;
          (Stmt::WireDecl {
            name,
            expr: Some(expr),
          }, end)
        } else {
          (Stmt::WireDecl {
            name,
            expr: None,
          }, maybe_end)
        }
      },
      
      Token::Name(kw) if kw == "inst" => {
        let name = tokens.next_identifier()?.0;
        let args = parse_list_paren_comma(tokens, Expr::parse)?;
        let end = tokens.peek_assert(&Token::Semicolon)?;
        (Stmt::ModuleInst {
          module: name,
          args,
        }, end)
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
        }, end)
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
pub enum SignalClass {
  Single,
  Mixed
}

impl SignalClass {
  pub fn parse(tokens: &Cursor) -> Result<(Self, Span), CerrSpan> {
    tokens.next_map(|v| {
      Ok(match v {
        Token::Name(name) if name == "single" => SignalClass::Single,
        Token::Name(name) if name == "mixed" => SignalClass::Mixed,
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
          if tokens.peek()?.0 == &Token::LParen {
            tokens.next()?;
            // function call
            if tokens.peek()?.0 == &Token::RParen {
              Expr::FnCall {
                func: name.clone(),
                args: vec![],
              }
            } else {
              // parse arguments
              let mut args = vec![];
              loop {
                args.push(Expr::parse(tokens)?);
                let (t, span) = tokens.next()?;
                match t {
                  Token::RParen => break,
                  Token::Comma => continue,
                  _ => return Err(Cerr::UnexpectedToken(vec![")".into(), ",".into()]).with(span))
                }
              }
              Expr::FnCall {
                func: name.clone(),
                args,
              }
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
          tokens.next_assert(&Token::RBrace)?;
          expr
        }
        _ => return Err(Cerr::InvalidExpr.with(span))
      }
    } else {
      let car = Expr::parse(tokens)?;
      let mut cdr = vec![];
      while matches!(tokens.peek()?.0, Token::Op(op) if op.precedence() == prec) {
        // unwrap: can only be op if the above matches! passes
        let op = tokens.next()?.0.get_op().unwrap();
        let expr = Expr::parse(tokens)?;
        cdr.push((op, expr));
      }
      Expr::BinaryOps {
        car: Box::new(car),
        cdr,
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
  loop {
    elems.push(f(tokens)?);
    let (token, span ) = tokens.next()?;
    if token == end { break }
    if token == delim {
      // to handle trailing commas
      let peek = tokens.peek()?.0;
      if peek == end { break }
      continue
    }
    return Err(Cerr::UnexpectedToken(vec![end.to_string(), delim.to_string()]).with(span));
  }
  Ok(elems)
}