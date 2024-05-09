# FHDL Compiler Internals

Basic process:
- Parsing
  - Tokenization
  - AST
- Synth
  - IR Transformation
  - Presynthesis
  - Synthesis
  - Optimization
- Layout
  - ?

This is a relatively quick walkthrough of the entire compiler.

The code starts at the parsing stage. The string is fed into a `Tokenizer`, which creates an iterator of
`Token`s. Since FHDL is context-free, it would theoretically be possible to parse the token iterator
directly without any extra state. But to keep things simple, the parser just collects all the tokens into
a `TokenStream`, which is just an array of tokens. `Cursor`s are used to view the tokens, each `Cursor` being
a separate stream into the `TokenStream` array. This was done so that `Cursor`s could easily backtrack
or lookahead any number of tokens.

The AST stage is relatively straightforwards; all AST structs have a `parse` method that accepts a `Cursor`
and attempts to parse their data structure starting at that `Cursor`.

Throughout the parsing stage, `Span` information is retained. The tokenizer actually emits a tuple of `(Token, Span)`,
and these `Span`s are used to construct pretty error messages by embedding them into a `CerrSpan`. At the end
of the AST stage, the only `Span` information that is lost is `Span`s within parts of expressions, because
after AST, it is sufficient to tell the user what statement the error occurs in rather than the exact expression.
At this stage, values are just AST expressions.

Synthesis takes place in a few phases. First, the AST is transformed into IR. The IR transformation flattens
expressions, does type checking, and transforms triggers into special assignments. Values in IR are represented
with `IRValue`s, which can either be nets, literals, or strings. During expression flattening, IR transformation
will create new nets and refer to them by their string name as well as keeping variables by name. IR is still
mostly a source-code abstraction.

After IR is the presynthesis step. During this stage, All IR structures are converted into `IncompleteNet`s and
`IncompleteCombinator`s. `IncompleteNet`s abstractly represent nets. They do not have an assigned wire colour or
signal type. `IncompleteCombinator`s wrap a `Combinator` with some info to enable the synthesis stage to convert
their references to `IncompleteNet`s to `Net`s.

`BuiltinFunction` is a trait that synthesizers for built-in functions implement.
The presynth stage uses a registry of `BuiltinFunction`s to convert `IRStmt`s into `IncompleteCombinators` by
calling their `synthesize` method. All binary operations (e.g. `+ - / *`) are synthesized using specially named
`BuiltinFunction`s.

Next is the synthesis step. Synthesis is straightforward, convert all `IncompleteNet`s and `IncompleteCombinator`s into
`Net`s and `Combinator`s and add them to the `NetList`. Note that each `IncompleteNet` expands into two `Net`s, one
for red and one for the green wire. The reason for making `IncompleteNet`s and `IncompleteCombinator`s
instead of just creating `Net`s and `Combinators`s is so that signals can be assigned to `IncompleteNet`s without possibility of conflict.

A to be implemented part of synthesis is optimization. The purpose of optimization is to fix known inefficiencies
with the synth process. One is to remove unnecessary
passthrough combinators created during synthesis. Since synthesis goes one module at a time, it is
unable to determine if tying nets together is safe or not, since passthrough combinators act as one-way gates for
signal propagation. Another one is to merge constant combinators. Passthrough combinators with literal arguments
get presynthesized into constant combinators, but merging them would require at least looking outside the current
statement. Better results can be had from looking at the entire netlist, so that's what this optimization pass does.

Optimization passes are implemented with the `OptimizePass` trait, which simply gives you the netlist to do
whatever with.

The final step is layout. I haven't finished this step yet.