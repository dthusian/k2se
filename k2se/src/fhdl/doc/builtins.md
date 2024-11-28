# List of Built-in Functions

> Note: if you see a built-in whose name starts with a '$', it means it's
> only for internal use.

Notation for argument types:
- `any` - Anything except a string.
- `mixed` - Mixed net.
- `single` - Single net.
- `singlelit` - Single net or int literal.
- `string` - String literal.

Notation for return types:
- `single` - Single net.
- `mixed` - Mixed net.

## Triggers

These builtins are internally used by trigger blocks to convert them into a
"raw" trigger. Each of these functions compares the input from this tick with the input from last tick,
and if the trigger condition is met, it outputs 0. Otherwise, it outputs 1. Note that if the condition is satisfied for
multiple ticks, the output will be 0 for every tick where the condition is true.

- `trig_inc(single) -> single` - Triggers on increasing
- `trig_dec(single) -> single` - Triggers on decreasing
- `trig_chg(single) -> single` - Triggers on any change