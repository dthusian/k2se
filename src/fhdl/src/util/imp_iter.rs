use std::iter::Peekable;

/// An iterator that supports an imperative iterator style.
/// Useful for parsers, where there is strongly heterogenous iterator use.
pub struct ImperativeIterator<I: Iterator> {
  i: Peekable<I>
}

impl<I: Iterator> Iterator for ImperativeIterator<I> {
  type Item = I::Item;

  fn next(&mut self) -> Option<Self::Item> {
    self.i.next()
  }
}

impl<I: Iterator> ImperativeIterator<I> {
  /// ImperativeIterator holds a Peekable under the hood to support its various methods.
  /// This method allows users to access the Peekable.
  pub fn peek(&mut self) -> Option<&I::Item> {
    self.i.peek()
  }
  
  /// Takes elements while a predicate is true and returns a Vec.
  /// Named `imp_take_while` to avoid conflicts with [`Iterator::take_while`].
  pub fn imp_take_while(&mut self, mut pred: impl FnMut(&I::Item) -> bool) -> Vec<I::Item> {
    let mut buf = vec![];
    loop {
      if let Some(peek) = self.peek() {
        if pred(peek) {
          // unwrap: if peek returns Some, there will be something to take
          let next = self.next().unwrap();
          buf.push(next);
        } else {
          break;
        }
      } else {
        break;
      }
    }
    buf
  }
  
  /// Takes the next element if it is equal to the given element.
  pub fn next_if_eq(&mut self, cmp: &I::Item) -> Option<I::Item> where I::Item: PartialEq {
    let peek = self.i.peek();
    if peek.map(|v| v == cmp).unwrap_or(false) {
      self.i.next()
    } else {
      None
    }
  }
  
  /// Takes a fixed number of elements (or however many are left) and returns it in a Vec.
  pub fn take_n(&mut self, n: usize) -> Vec<I::Item> {
    let mut b = vec![];
    for _ in 0..n {
      if let Some(el) = self.i.next() {
        b.push(el);
      } else {
        break
      }
    }
    b
  }
}

pub fn imperative<I: Iterator>(i: I) -> ImperativeIterator<I> {
  ImperativeIterator { i: i.peekable() }
}