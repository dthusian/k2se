# FHDL Guide

This guide assumes a basic understanding of Factorio circuit networks and HDLs.

## Signals

Signals in FHDL are seperated into two types: `single` and `mixed`. `single` signals
represent a single signal of some type that is controlled by the sythesis engine,
while `mixed` signals represent wires that have multiple Factorio signal types on them.

In synthesis, multiple `single` wires will not be merged into one.

## Modules

An FHDL program is composed of modules, which represent groups of combinators.
A module uses wires to connect with other modules.

There are 3 types of wire connections: `in`, `out`, and `inout`. These connection
types are verified so that you cannot for example, write to an input wire.

A module is declared with the following syntax:

```
module <module name>(<connection type> <signal type> <wire name>, ...) {
  <stmts> ...
}
```

## Wires and Memory

The two types of objects that can be declared within a module are wires or memory.
They can be either single or mixed-signal.

Wires represent, well, wires. Their value solely depends on what was written to them last tick.

Memory represents memory cells. These are implemented in Factorio as self-feeding combinators
and as such can hold values across ticks.

Wires and memory are declared as follows:

```
wire <wire-name> [= <expr>];
mem <mem-name>; 
```

## Assignment and Trigger Blocks

A module is composed of statements which assign values to wires and memory based on the values
of other wires and memory. There are a few different ways to assign these values:

- `=`, which simply assigns values to objects
- `+=`, which increments the value of objects

However, not all assignment types are valid in all contexts. Memory cells cannot be continuously
`=`-assigned to.

By default, all statements in a module execute continuously, that is every Factorio tick. Statements in
trigger blocks only execute when the trigger condition is met. Trigger blocks have the following syntax:

```
trigger <identifier> <trigger-type> {
  <stmts> ...
}
```

`<trigger-type>` is one of `increasing`, `decreasing`, `changed`, or `raw`. Note that since a trigger block
checks its condition every tick, if a wire increases over multiple ticks, a corresponding
`trigger <wire> increasing` will execute multiple times.

`raw` trigger mode checks if the targeted signal equals zero. Internally, all other trigger modes
are synthesized as rising or falling edge detectors that set a signal to zero when the condition
is met.

## Instantiating Modules

Modules can be instantiated to create copies of them within other modules. This is done as follows:
```
inst <module-name>(<args> ...);
```

## Output Mixing

Just like in real circuits, having multiple entities output to the same wire is usually
undesired. Thus, for any wire, it can only be `=`-assigned once or be passed to a module instatiation
as an `out` connection once.

If output mixing is desired, then it can be achieved with `+=`-assignment or with `inout` module
connections.

## Operators

The following built-in operators are provided. Note that all arithmetic is done on 32-bit signed
values since that is what Factorio uses for signals.

In order of precedence:

- `**`: Exponentiation
- `*`, `/`, `%`: Signed multiply/divide/remainder
- `+`, `-`: Add/subtract
- `<<`, `>>`: Left shift and arithmetic right shift
- `&`, `|`, `^`: Bitwise AND, OR, XOR
- `==`, `!=`, `<=`, `>=`, `<`, `>`: Comparison operators. Output 1 on true, 0 otherwise.

## Built-in Functions

The following built-in functions are provided.