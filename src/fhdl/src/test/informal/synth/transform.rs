use crate::parse::ast::Program;
use crate::parse::tokenizer::tokenize;
use crate::parse::tokenstream::TokenStream;
use crate::synth::builtins::collect_builtins;
use crate::synth::ir::IRModule;
use crate::synth::transform::transform_modules;
use crate::util::ResultExt;

fn driver(filename: &str, src: &str) -> Vec<IRModule> {
  let tokens = tokenize(src.chars())
    .collect::<Result<Vec<_>, _>>()
    .pretty_unwrap();
  let token_stream = TokenStream::from_tokens(tokens);
  let program = Program::parse(&token_stream.begin()).pretty_unwrap();
  let transform = transform_modules(&program.modules, &collect_builtins());
  if transform.1.is_empty() {
    transform.0
  } else {
    let splitted = src.lines().collect::<Vec<_>>();
    for v in transform.1 {
      eprintln!("{}", v.format_err(filename, &splitted).pretty_unwrap());
    }
    panic!("Transform failed");
  }
}

#[test]
pub fn transform_counter() {
  let modules = driver(
    "counter.fhdl",
    include_str!("../../../../examples/counter.fhdl"),
  );
  eprintln!("{:?}", modules);
}
