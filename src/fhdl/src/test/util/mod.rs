use crate::util::imp_iter::imperative;

#[test]
pub fn imp_iter_peek() {
  let mut iter = imperative(vec![1, 1, 2, 3, 5].into_iter());
  iter.next();
  iter.next();
  assert_eq!(iter.peek(), Some(&2));
  assert_eq!(iter.peek(), Some(&2));
}

#[test]
pub fn imp_iter_take_while() {
  let mut iter = imperative(vec![1, 1, 2, 3, 5, 0, 7, 8, 9, 0].into_iter());
  let elements = iter.imp_take_while(|v| *v > 0);
  assert_eq!(elements, vec![1, 1, 2, 3, 5]);
  let elements = iter.imp_take_while(|v| *v > 0);
  assert_eq!(elements, vec![]);
  assert_eq!(iter.next(), Some(0));
  let elements = iter.imp_take_while(|v| *v > 0);
  assert_eq!(elements, vec![7, 8, 9]);
}

#[test]
pub fn imp_iter_take_n() {
  let mut iter = imperative(vec![1, 1, 2, 3, 5, 0, 7, 8, 9, 0].into_iter());
  let elements = iter.take_n(4);
  assert_eq!(elements, vec![1, 1, 2, 3]);
  let elements = iter.take_n(4);
  assert_eq!(elements, vec![5, 0, 7, 8]);
  let elements = iter.take_n(4);
  assert_eq!(elements, vec![9, 0]);
}