use std::fmt::{Display, Formatter, Write};
use std::iter::Peekable;
use std::str::FromStr;
use crate::err::{Cerr, CerrSpan};
use crate::util::imp_iter::{imperative, ImperativeIterator};
use crate::parse::iter_with_pos::{with_pos, WithPos};
use crate::parse::span::{Pos, Span, WithSpan};

/// Represents a token.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Token {
  Name(String),
  Literal(i32),
  LParen,
  RParen,
  LBrace,
  RBrace,
  Comma,
  Semicolon,
  Op(BinaryOp)
}

impl Display for Token {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Token::Name(s) => f.write_str(s),
      Token::Literal(i) => write!(f, "{}", i),
      Token::LParen => f.write_char('('),
      Token::RParen => f.write_char(')'),
      Token::LBrace => f.write_char('{'),
      Token::RBrace => f.write_char('}'),
      Token::Comma => f.write_char(','),
      Token::Semicolon => f.write_char(';'),
      Token::Op(op) => write!(f, "{}", op),
    }
  }
}

impl Token {
  /// Convenience method for creating a [`Token::Name`] with a `&str`
  pub fn name(s: &str) -> Self {
    Token::Name(s.into())
  }
  
  pub fn get_name(&self) -> Option<&str> {
    match self {
      Token::Name(s) => Some(s.as_str()),
      _ => None
    }
  }
  
  pub fn get_literal(&self) -> Option<i32> {
    match self {
      Token::Literal(x) => Some(*x),
      _ => None,
    }
  }
  
  pub fn get_op(&self) -> Option<BinaryOp> {
    match self {
      Token::Op(op) => Some(*op),
      _ => None
    }
  }
}

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
  Ge,
  Assign,
  AddAssign,
}

const HIGHEST_PREC: u32 = 6;

impl Display for BinaryOp {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      BinaryOp::Add => "+",
      BinaryOp::Sub => "-",
      BinaryOp::Mul => "*",
      BinaryOp::Div => "/",
      BinaryOp::Mod => "%",
      BinaryOp::Pow => "**",
      BinaryOp::And => "&",
      BinaryOp::Or => "|",
      BinaryOp::Xor => "^",
      BinaryOp::Shl => "<<",
      BinaryOp::Shr => ">>",
      BinaryOp::Eq => "==",
      BinaryOp::Ne => "!=",
      BinaryOp::Lt => "<",
      BinaryOp::Gt => ">",
      BinaryOp::Le => "<=",
      BinaryOp::Ge => ">=",
      BinaryOp::Assign => "=",
      BinaryOp::AddAssign => "+="
    })
  }
}

impl BinaryOp {
  pub fn parse_raw(s: &str) -> Result<Self, Cerr> {
    Ok(match s {
      "+" => BinaryOp::Add,
      "-" => BinaryOp::Sub,
      "*" => BinaryOp::Mul,
      "/" => BinaryOp::Div,
      "%" => BinaryOp::Mod,
      "**" => BinaryOp::Pow,
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
      "=" => BinaryOp::Assign,
      "+=" => BinaryOp::AddAssign,
      _ => return Err(Cerr::InvalidOperator)
    })
  }
}

pub struct Tokenize<I: Iterator<Item = char>> {
  i: ImperativeIterator<Peekable<WithPos<I>>>,
}

impl<I: Iterator<Item = char>> Tokenize<I> {
  fn _next(&mut self) -> Option<(Pos, char)> {
    self.i.next()
  }
  
  fn _peek(&mut self) -> Option<(Pos, char)> {
    self.i.peek().copied()
  }
  
  /// Reads as many characters as possible that satisfy a predicate.
  /// Returns the position of the last taken character. Returns `None`
  /// if nothing was read.
  fn take_while_span(&mut self, mut pred: impl FnMut(char) -> bool) -> Option<(Span, String)> {
    let buf = self.i.imp_take_while(|(_, c)| pred(*c));
    if buf.is_empty() {
      None
    } else {
      Some((
        Span {
          start: buf[0].0,
          end: buf[buf.len() - 1].0,
        },
        buf.iter().map(|v| v.1).collect::<String>()
      ))
    }
  }
  
  /// Reads an name from the input stream. Panics if the stream is not at a name.
  fn parse_name(&mut self) -> Result<WithSpan<Token>, CerrSpan> {
    let (span, s) = self.take_while_span(|c| is_ident(c))
      .expect("Not a name");
    Ok(WithSpan {
      span,
      t: Token::Name(s),
    })
  }
  
  /// Reads an integer literal from the input stream. Returns a Result if integer parsing failed.
  /// Panics if the stream is not at an integer literal.
  fn parse_literal(&mut self) -> Result<WithSpan<Token>, CerrSpan> {
    let (span, s) = self.take_while_span(|c| is_ident(c))
      .expect("Not a literal");
    let discrim = s.chars().nth(1);
    let is_hex = discrim.map(|v| v == 'x' || v == 'X').unwrap_or(false);
    let is_bin = discrim.map(|v| v == 'b' || v == 'B').unwrap_or(false);
    let res = if is_hex {
      let buf = s.chars().skip(2).filter(|v| *v != '_').collect::<String>();
      i64::from_str_radix(&buf, 16)
    } else if is_bin {
      let buf = s.chars().skip(2).filter(|v| *v != '_').collect::<String>();
      i64::from_str_radix(&buf, 16)
    } else {
      i64::from_str(&s)
    }
      .map_err(|v| v.into() )
      .map(|v| Token::Literal(v as i32));
    util_inject_span(res, span)
  }
  
  /// Reads an operator or skips a comment. Returns None if a comment was matched.
  fn parse_op_or_comment(&mut self) -> Option<Result<WithSpan<Token>, CerrSpan>> {
    let (span, s) = self.take_while_span(|c| is_op(c))
      .expect("Not an operator");
    if s.starts_with("//") {
      self.take_while_span(|c| c != '\n');
      None
    } else {
      Some(util_inject_span(BinaryOp::parse_raw(&s).map(Token::Op), span))
    }
  }
}

impl<I: Iterator<Item = char>> Iterator for Tokenize<I> {
  type Item = Result<WithSpan<Token>, CerrSpan>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let (pos, peek) = self._peek()?;
      if is_ident_start(peek) {
        return Some(self.parse_name())
      }
      if char::is_ascii_digit(&peek) {
        return Some(self.parse_literal())
      }
      if char::is_ascii_whitespace(&peek) {
        self._next();
        continue;
      }
      if is_op(peek) {
        if let Some(op) = self.parse_op_or_comment() {
          return Some(op)
        } else {
          continue;
        }
      }
      let (_, next) = self._next().unwrap();
      return Some(match peek {
        '{' => Ok(Token::LBrace),
        '}' => Ok(Token::RBrace),
        '(' => Ok(Token::LParen),
        ')' => Ok(Token::RParen),
        ';' => Ok(Token::Semicolon),
        ',' => Ok(Token::Comma),
        _ => Err(CerrSpan::new(pos.into(), Cerr::InvalidChar)),
      }.map(|v| WithSpan::new(pos.into(), v)))
    }
  }
}

pub fn tokenize<I: Iterator<Item = char>>(iter: I) -> Tokenize<I> {
  return Tokenize {
    i: imperative(with_pos(iter).peekable()),
  }
}

/// Matches characters that can be the start of an identifier.
fn is_ident_start(c: char) -> bool {
  c.is_ascii_alphabetic() || c == '_'
}

/// Matches characters that can be in an identifier.
fn is_ident(c: char) -> bool {
  c.is_ascii_alphanumeric() || c == '_'
}

/// Matches characters that make operators.
fn is_op(c: char) -> bool {
  match c {
    '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '=' | '!' | '<' | '>' => true,
    _ => false
  }
}

/// TokenKind and Cerr are similar in that there exist two other structs
/// (Token, CerrSpan) that wrap the token and a span.
fn util_inject_span(r: Result<Token, Cerr>, span: Span) -> Result<WithSpan<Token>, CerrSpan> {
  r.map(|v| WithSpan::new(span, v))
    .map_err(|v| CerrSpan::new(span, v))
}