use std::cmp::Ordering;

/// A position in source code.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Pos {
  pub line: u32,
  pub col: u32,
}

impl PartialOrd<Self> for Pos {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(<Self as Ord>::cmp(self, other))
  }
}

impl Ord for Pos {
  fn cmp(&self, other: &Self) -> Ordering {
    let ord = self.line.cmp(&other.line);
    if ord == Ordering::Equal {
      self.col.cmp(&other.col)
    } else {
      ord
    }
  }
}

impl Default for Pos {
  fn default() -> Self {
    Pos::new(1, 0)
  }
}

impl Pos {
  pub fn new(line: u32, col: u32) -> Pos {
    Pos { line, col }
  }
}

/// A span represents a section of source code, delimited by two positions.
/// Spans are inclusive ranges and so include both endpoints.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct Span {
  pub start: Pos,
  pub end: Pos,
}

impl PartialOrd<Self> for Span {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(<Self as Ord>::cmp(self, other))
  }
}

impl Ord for Span {
  fn cmp(&self, other: &Self) -> Ordering {
    let ord = self.start.cmp(&other.start);
    if ord == Ordering::Equal {
      self.end.cmp(&other.end)
    } else {
      ord
    }
  }
}

impl Span {
  /// Merges two spans. If they aren't adjacent or don't overlap,
  /// the area in between the two spans is considered to be part
  /// of the union.
  pub fn union(self, other: Span) -> Span {
    Span {
      start: self.start.min(other.start),
      end: self.end.max(other.end),
    }
  }
}

impl From<Pos> for Span {
  fn from(value: Pos) -> Self {
    Span {
      start: value,
      end: value,
    }
  }
}

/// A struct that wraps another type in a Span.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct WithSpan<T> {
  pub span: Span,
  pub t: T,
}

impl<T> WithSpan<T> {
  pub fn new(span: Span, t: T) -> Self {
    WithSpan { span, t }
  }
}
