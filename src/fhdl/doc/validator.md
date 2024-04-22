# Validator Errors

The validators must catch the following errors:
- Multiple/no declaration of an object
- Writes to input wire
- Multiple assigns to one wire if it has been `=`-assigned
- Nested trigger blocks
- Non-set items in a trigger block (todo)
- `=`-assign to memory outside of trigger blocks
- Module not found
- Module arity mismatch

The synth code should be able to ignore these errors.