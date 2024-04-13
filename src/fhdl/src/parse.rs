use std::error::Error;
use nom::bytes::complete::{is_not, tag, take_until, take_while};
use nom::error::{ParseError, VerboseError};
use nom::{IResult, Parser};
use nom::branch::alt;
use nom::character::complete::{alpha1, alphanumeric1, char, multispace0, multispace1, one_of};
use nom::combinator::{cut, map_res, recognize};
use nom::multi::{many0, many0_count, many1, separated_list0};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};

macro_rules! curry {
    ($f:expr,$arg:expr) => {
      |res| $f($arg, res)
    };
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Program {
  pub version: String,
  pub modules: Vec<Module>
}

impl Program {
  pub fn parser(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
    pair(
      delimited(
        wc01(tag("version")),
        wc(identifier),
        wc(tag(";"))
      ),
      many0(Module::parser)
    )
      .map(|(version, modules)| Program {
        version: version.to_owned(),
        modules,
      })
      .parse(s)
  }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Module {
  pub name: String,
  pub ports: Vec<PortDecl>,
  pub stmts: Vec<Stmt>
}

impl Module {
  pub fn parser(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
    tuple((
      preceded(
        ws01(tag("module")),
        wc(identifier),
      ),
      delimited(
        wc(tag("(")),
        separated_list0(wc(tag(",")), cut(PortDecl::parser)),
        wc(tag(")")),
      ),
      delimited(
        wc(tag("{")),
        many0(wc(Stmt::parser)),
        wc(tag("}")),
      ),
    ))
      .map(|(name, ports, stmts)| Module {
        name: name.to_owned(),
        ports,
        stmts,
      })
      .parse(s)
  }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PortDecl {
  pub port_class: PortClass,
  pub signal_class: SignalClass,
  pub name: String,
}

impl PortDecl {
  pub fn parser(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
    tuple((ws01(PortClass::parser), ws01(SignalClass::parser), wc(identifier)))
      .map(|(port_class, signal_class, name)| PortDecl {
        port_class,
        signal_class,
        name: name.to_owned(),
      })
      .parse(s)
  }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum PortClass {
  In, Out, InOut,
}

impl PortClass {
  pub fn parser(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
    map_res(
      take_while(char::is_alphabetic),
      |v| {
        Ok(match v {
          "in" => PortClass::In,
          "out" => PortClass::Out,
          "inout" => PortClass::InOut,
          _ => return Err("Expected `in`, `out`, or `inout`")
        })
      }).parse(s)
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
  pub fn parser(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
    todo!()
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
  pub fn parser(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
    map_res(
      take_while(char::is_alphabetic),
      |v| {
        Ok(match v {
          "increasing" => TriggerKind::Increasing,
          "decreasing" => TriggerKind::Decreasing,
          "changed" => TriggerKind::Changed,
          "raw" => TriggerKind::Raw,
          _ => return Err::<Self, Box<dyn Error>>("Expected `increasing`, `decreasing`, `changed`, or `raw`".into()),
        })
      }).parse(s)
  }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum SignalClass {
  Single,
  Mixed
}

impl SignalClass {
  pub fn parser(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
    map_res(
      take_while(char::is_alphabetic),
      |v| {
        Ok(match v {
          "single" => SignalClass::Single,
          "mixed" => SignalClass::Mixed,
          _ => return Err::<Self, Box<dyn Error>>("Expected `single` or `mixed`".into())
        })
      }).parse(s)
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
    precedence: u32,
    car: Box<Expr>,
    cdr: Vec<(BinaryOp, Expr)>
  },
  Braced {
    inner: Box<Expr>
  },
}

impl Expr {
  pub fn parser(s: &str) -> IResult<&str, Self, VerboseError<&str>> {
    Self::parser_with_prec(HIGHEST_PREC, s)
  }
  
  pub fn parser_with_prec(prec: u32, s: &str) -> IResult<&str, Self, VerboseError<&str>> {
    switch(
      prec > 0,
      pair(
        wc(curry!(Expr::parser_with_prec, prec - 1)),
        many0(pair(
          wc(curry!(BinaryOp::parser, prec)),
          wc(curry!(Expr::parser_with_prec, prec - 1))
        ))
      ).map(|(car, cdr)| if cdr.is_empty() {
        car
      } else {
        Expr::BinaryOps {
          precedence: prec,
          car: Box::new(car),
          cdr,
        }
      }),
      alt((
        tuple((
          wc(identifier),
          delimited(
            wc(tag("(")),
            wc(separated_list0(wc(tag(",")), cut(wc(Expr::parser)))),
            wc(tag(")"))
          ),
        )).map(|(func, args)| Expr::FnCall {
          func: func.to_owned(),
          args,
        }),
        delimited(
          wc(tag("(")),
          wc(Expr::parser),
          wc(tag(")"))
        ).map(|expr| Expr::Braced {
          inner: Box::new(expr),
        }),
        wc(parse_int)
          .map(|v| Expr::Literal { val: v }),
        wc(identifier)
          .map(|v| Expr::Identifier { name: v.to_owned(), })
      ))
    ).parse(s)
  }
}

const HIGHEST_PREC: u32 = 6;

/// Operator Precedence (high number = eval last):
/// Within same precedence class, operators are eval'd left to right
/// 1: Pow
/// 2: Div, Mul, Mod
/// 3: Add, Sub
/// 4: Shl, Shr
/// 5: And, Or, Xor
/// 6: Eq, Ne, Lt, Gt, Le, Ge
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BinaryOp {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Pow,
  And,
  Or,
  Xor,
  Shl,
  Shr,
  Eq,
  Ne,
  Lt,
  Gt,
  Le,
  Ge
}

impl BinaryOp {
  pub fn parse_raw(s: &str) -> Result<Self, Box<dyn Error>> {
    Ok(match s {
      "+" => BinaryOp::Add,
      "-" => BinaryOp::Sub,
      "*" => BinaryOp::Mul,
      "/" => BinaryOp::Div,
      "%" => BinaryOp::Mod,
      "&" => BinaryOp::And,
      "|" => BinaryOp::Or,
      "^" => BinaryOp::Xor,
      "<<" => BinaryOp::Shl,
      ">>" => BinaryOp::Shr,
      "==" => BinaryOp::Eq,
      "!=" => BinaryOp::Ne,
      "<" => BinaryOp::Lt,
      ">" => BinaryOp::Gt,
      "<=" => BinaryOp::Le,
      ">=" => BinaryOp::Ge,
      _ => return Err("Invalid operator".into())
    })
  }
  pub fn parser(prec: u32, s: &str) -> IResult<&str, Self, VerboseError<&str>> {
    map_res(alt((
      switch(prec == 6, alt((tag("=="), tag("!="), tag("<="), tag(">="), tag("<"), tag(">"))), nothing),
      switch(prec == 5, alt((tag("&"), tag("|"), tag("^"))), nothing),
      switch(prec == 4, alt((tag("<<"), tag(">>"))), nothing),
      switch(prec == 3, alt((tag("+"), tag("-"))), nothing),
      switch(prec == 2, alt((tag("*"), tag("/"), tag("%"))), nothing),
      switch(prec == 1, alt((tag("**"),)), nothing)
    )), Self::parse_raw).parse(s)
  }
}

pub fn identifier(s: &str) -> IResult<&str, &str, VerboseError<&str>> {
  recognize(
    pair(
      alt((alpha1, tag("_"))),
      many0_count(alt((alphanumeric1, tag("_"))))
    )
  ).parse(s)
}

/// Stolen from `nom` recipes guide.
/// 
/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and 
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
  where
    F: Parser<&'a str, O, E>,
{
  delimited(
    multispace0,
    inner,
    multispace0
  )
}

/// Same as `ws`, but trailing whitespace is required
fn ws01<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
  where
    F: Parser<&'a str, O, E>,
{
  delimited(
    multispace0,
    inner,
    multispace1
  )
}

/// A combinator that wraps another one and removes either whitespace or comments
fn wc<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
  where
    F: Parser<&'a str, O, E>,
{
  delimited(
    many0(alt((multispace0.map(|_| ()), comment))),
    inner,
    many0(alt((multispace0.map(|_| ()), comment)))
  )
}

/// Same as `wc`, but trailing whitespace or comments is required
fn wc01<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
  where
    F: Parser<&'a str, O, E>,
{
  delimited(
    many0(alt((multispace0.map(|_| ()), comment))),
    inner,
    many1(alt((multispace0.map(|_| ()), comment)))
  )
}


/// Matches a C-style comment.
pub fn comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
  alt((
    delimited(
      tag("/*"),
      take_until("*/"),
      tag("*/")
    ),
    preceded(tag("//"), is_not("\n"))
  ))
    .map(|_| ())
    .parse(i)
}

/// Switches the matching depending on a runtime bool
pub fn switch<I, O, E>(sw: bool, mut p1: impl Parser<I, O, E>, mut p2: impl Parser<I, O, E>) -> impl Parser<I, O, E> {
  move |s: I| if sw {
    p1.parse(s)
  } else {
    p2.parse(s)
  }
}

/// Matches an integer literal. Stolen from nom recipes.
pub fn parse_int(s: &str) -> IResult<&str, i32, VerboseError<&str>> {
  alt((
    map_res(preceded(
      alt((tag("0x"), tag("0X"))),
      recognize(
        many1(
          terminated(one_of("0123456789abcdefABCDEF"), many0(char('_')))
        )
      )
    ), |out: &str| i64::from_str_radix(&str::replace(&out, "_", ""), 16)),
    map_res(preceded(
      alt((tag("0b"), tag("0B"))),
      recognize(
        many1(
          terminated(one_of("01"), many0(char('_')))
        )
      )
    ), |out: &str| i64::from_str_radix(&str::replace(&out, "_", ""), 2)),
    map_res(recognize(
      many1(
        terminated(one_of("0123456789"), many0(char('_')))
      )
    ), |out: &str| i64::from_str_radix(&str::replace(&out, "_", ""), 10))
  ))
    .map(|v| v as i32)
    .parse(s)
}

pub fn nothing<'a, O, E: ParseError<&'a str>>(s: &'a str) -> IResult<&'a str, O, E> {
  one_of("").map(|_| unreachable!()).parse(s)
}