use crate::parse::ast::Program;
use crate::parse::tokenizer::tokenize;
use crate::parse::tokenstream::TokenStream;
use crate::synth::builtins::collect_builtins;
use crate::synth::combinator::{Signal, SignalType};
use crate::synth::netlist::Netlist;
use crate::synth::synth::{synthesize, SynthSettings};
use crate::synth::transform::transform_modules;
use crate::util::ResultExt;

fn driver(filename: &str, src: &str, synth_settings: &SynthSettings) -> Netlist {
  let tokens = tokenize(src.chars())
    .collect::<Result<Vec<_>, _>>()
    .pretty_unwrap();
  let token_stream = TokenStream::from_tokens(tokens);
  let program = Program::parse(&token_stream.begin()).pretty_unwrap();
  let transform = transform_modules(&program.modules, &collect_builtins());
  if transform.1.is_empty() {
    synthesize(synth_settings, &transform.0, &collect_builtins()).pretty_unwrap()
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
    &SynthSettings {
      main: "counter".into(),
      main_module_conn_names: vec![
        ['R', 'S', 'T', ' '],
        ['C', 'N', 'T', ' ']
      ],
      main_module_conn_signals: vec![
        Signal {
          ty: SignalType::Virtual,
          name: "signal-R".into(),
        },
        Signal {
          ty: SignalType::Virtual,
          name: "signal-C".into(),
        }
      ],
    }
  );
  eprintln!("{:#?}", modules);
}
