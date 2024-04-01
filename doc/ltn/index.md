# How to use LTN station blueprints

Item/Fluid Provider:
- "Limit trains" is how many trains the station can support. Should be 3.
- "Provide threshold" is how many items/fluid units fit on a train.

Item Requester:
- "Limit trains" is how many trains the station can support. Should be 3.
- "Provide threshold" is some arbitrarily high number at least 2x the request amount.
- "Request threshold" is how many item/fluid units fit on a train.
- Set the requested item to its requested amount, but negative.
  - e.g. Request of 100k iron plate is represented by -100k iron plate in constant combinator.
  - The default request in the bp is fish.

LTN will try to keep the number of items in the chest between `Request Amount` and `Request Amount - Request Threshold`.

80 slots per train (40 per wagon * 2 wagons).

Quick ref for item train sizes:
- Stack 10: 8k
- Stack 20: 16k
- Stack 50: 40k
- Stack 100: 80k
- Stack 200: 160k