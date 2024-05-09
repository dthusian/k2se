use std::borrow::Cow;
use crate::synth::combinator::{Signal, SignalType};

const fn cow(s: &'static str) -> Cow<'static, str> {
  Cow::Borrowed(s)
}

pub const VIRTUAL_SIGNALS: [Signal; 36] = [
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-A"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-B"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-C"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-D"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-E"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-F"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-G"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-H"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-I"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-J"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-K"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-L"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-M"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-N"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-O"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-P"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-Q"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-R"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-S"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-T"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-U"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-V"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-W"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-X"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-Y"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-Z"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-0"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-1"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-2"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-3"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-4"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-5"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-6"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-7"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-8"),
  },
  Signal {
    ty: SignalType::Virtual,
    name: cow("signal-9"),
  },
];