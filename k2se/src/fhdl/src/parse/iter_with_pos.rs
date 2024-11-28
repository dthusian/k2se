use crate::parse::span::Pos;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WithPos<I: Iterator<Item = char>> {
  i: I,
  pos: Pos,
}

impl<I: Iterator<Item = char>> Iterator for WithPos<I> {
  type Item = (Pos, char);

  fn next(&mut self) -> Option<Self::Item> {
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
}

pub fn with_pos<I: Iterator<Item = char>>(i: I) -> WithPos<I> {
  WithPos {
    i,
    pos: Pos::new(1, 0),
  }
}
