# How to use LTN station blueprints

Item/Fluid Provider:
- "Limit trains" is how many trains the station can support. Should be 3.
- "Provide threshold" is how many items/fluid units fit on a train.

Item Requester:
- "Limit trains" is how many trains the station can support. Should be 3.
- "Provide threshold" is some arbitrarily high number at least 2x the request amount.
- "Request threshold" is how many item/fluid units fit on a train.
- Set the requested item to `-4x`, where `x` is the number of units on a train.
  - e.g. Request of iron plates is represented by -32k iron plate (since 1 train = 8k) in constant combinator.
  - The default request in the bp is fish.

LTN will try to keep the number of items in the chest between `Request Amount` and `Request Amount - Request Threshold`.

80 slots per train (40 per wagon * 2 wagons).

Quick ref for item train sizes/requests:
- Stack 10: 800, 3200
- Stack 20: 1600, 6400
- Stack 50: 4k, 16k
- Stack 100: 8k, 32k
- Stack 200: 16k, 64k

## Undefined Behavior

LTN is an incredibly dumb mod. It will just do things as it does and your blueprints need to deal with its consequences.
As a result, doing any of the following is UB (i.e., not supported by the blueprint) and will cause reduced train performance
or deadlocks.

- Any trains getting deadlocked.
  - This is because LTN will remove requests if they are not fulfilled in time. However, this leaves the train full of items.
- Setting requested items/fluid higher than can be held in buffers.
- Enabling a station while there is no path to that station.
- Provider stations loading multiple item/fluid types.
- Having a station that is named `[item:buffer-chest]` but is not configured as an LTN depot.