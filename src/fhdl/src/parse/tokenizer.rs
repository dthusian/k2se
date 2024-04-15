use std::iter::Peekable;
use std::str::FromStr;
use crate::err::{Cerr, CerrSpan};
use crate::parse::span::{Pos, Span, WithSpan};

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
  TrigAssign
}


const HIGHEST_PREC: u32 = 6;

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
      "*=" => BinaryOp::TrigAssign,
      _ => return Err(Cerr::InvalidOperator)
    })
  }
}

pub struct Tokenize<I: Iterator<Item = char>> {
  i: Peekable<I>,
  pos: Pos
}

impl<I: Iterator<Item = char>> Tokenize<I> {
  fn next_pos(&mut self) -> Option<(Pos, char)> {
    let next = self.i.next();
    if let Some(next) = next {
      let pos = self.pos;
      if next == '\n' {
        self.pos.line += 1;
        self.pos.col = 0;
      } else {
        self.pos.col += 1;
      }
      Some((pos, next))
    } else {
      None
    }
  }
  
  fn peek_pos(&mut self) -> Option<(Pos, char)> {
    self.i.peek().map(|peek| (self.pos, *peek))
  }
  
  /// Reads as many characters as possible that satisfy a predicate.
  /// Returns the position of the last taken character. Returns `None`
  /// if nothing was read.
  fn take_while_pos(&mut self, mut pred: impl FnMut(char) -> bool) -> Option<(Span, String)> {
    let mut start_pos = None;
    let mut end_pos = None;
    let mut buf = String::new();
    loop {
      if let Some((_, peek)) = self.peek_pos() {
        if pred(peek) {
          // unwrap: if peek returns Some, there will be something to take
          let (new_pos, next) = self.next_pos().unwrap();
          buf.push(next);
          start_pos.get_or_insert(new_pos);
          end_pos = Some(new_pos);
        } else {
          break;
        }
      } else {
        break;
      }
    }
    // unwrap: if start_pos is Some, the end_pos is Some
    start_pos.map(|_| (Span { start: start_pos.unwrap(), end: end_pos.unwrap() }, buf))
  }
  
  /// Reads an name from the input stream. Panics if the stream is not at a name.
  fn parse_name(&mut self) -> Result<WithSpan<Token>, CerrSpan> {
    let (span, s) = self.take_while_pos(|c| is_ident(c))
      .expect("Not a name");
    Ok(WithSpan {
      span,
      t: Token::Name(s),
    })
  }
  
  /// Reads an integer literal from the input stream. Returns a Result if integer parsing failed.
  /// Panics if the stream is not at an integer literal.
  fn parse_literal(&mut self) -> Result<WithSpan<Token>, CerrSpan> {
    let (span, s) = self.take_while_pos(|c| is_ident(c))
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
    let (span, s) = self.take_while_pos(|c| is_op(c))
      .expect("Not an operator");
    if s.starts_with("//") {
      self.take_while_pos(|c| c != '\n');
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
      let (pos, peek) = self.peek_pos()?;
      if is_ident_start(peek) {
        return Some(self.parse_name())
      }
      if char::is_ascii_digit(&peek) {
        return Some(self.parse_literal())
      }
      if char::is_ascii_whitespace(&peek) {
        self.next_pos();
        continue;
      }
      if is_op(peek) {
        if let Some(op) = self.parse_op_or_comment() {
          return Some(op)
        } else {
          continue;
        }
      }
      let (_, next) = self.next_pos().unwrap();
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
    i: iter.peekable(),
    pos: Pos { line: 1, col: 0 },
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