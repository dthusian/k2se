struct S {
  i: i32,
  foo1: Vec<i32>,
  foo2: Vec<i32>,
}

fn make_vec<T>(base: T, mut f: impl FnMut() -> T) -> Result<Vec<i32>, ()> {
  Ok(vec![base, f()])
}

impl S {
  pub fn make_i32() -> i32 {
    4
  }
  
  pub fn new(m: &i32) -> Result<Self, ()> {
    let foo1 = make_vec(*m, S::make_i32)?;
    let foo2 = make_vec(4, S::make_i32)?;
    Ok(S {
      i: 4,
      foo1,
      foo2
    })
  }
}

fn main() {
    println!("Hello, world!");
}
