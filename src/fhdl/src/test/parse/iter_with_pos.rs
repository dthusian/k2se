use crate::parse::iter_with_pos::with_pos;
use crate::parse::span::Pos;

#[test]
pub fn test_iter_pos1() {
  let iter = with_pos("RAD\nIOH\nEAD\n".chars());
  let expected = vec![
    (Pos::new(1, 0), 'R'),
    (Pos::new(1, 1), 'A'),
    (Pos::new(1, 2), 'D'),
    (Pos::new(1, 3), '\n'),
    
    (Pos::new(2, 0), 'I'),
    (Pos::new(2, 1), 'O'),
    (Pos::new(2, 2), 'H'),
    (Pos::new(2, 3), '\n'),

    (Pos::new(3, 0), 'E'),
    (Pos::new(3, 1), 'A'),
    (Pos::new(3, 2), 'D'),
    (Pos::new(3, 3), '\n'),
  ];
  assert_eq!(expected, iter.collect::<Vec<_>>());
}

#[test]
pub fn test_iter_pos2() {
  let iter = with_pos("a \n\n\n \r\n\tb".chars());
  let expected = vec![
    (Pos::new(1, 0), 'a'),
    (Pos::new(1, 1), ' '),
    (Pos::new(1, 2), '\n'),
    (Pos::new(2, 0), '\n'),
    (Pos::new(3, 0), '\n'),
    (Pos::new(4, 0), ' '),
    (Pos::new(4, 1), '\r'),
    (Pos::new(4, 2), '\n'),
    (Pos::new(5, 0), '\t'),
    (Pos::new(5, 1), 'b'),
  ];
  assert_eq!(expected, iter.collect::<Vec<_>>());
}