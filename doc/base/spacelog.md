# Space Logistics Options

## Delivery Cannons

Easiest to set up and use. Delivery cannons shoot resources to a delivery cannon chest.
All fluids (in barrels) and only certain items can be transported.

> Warning: If the delivery cannon chest is full, it will cause damage.

## Uniform Cargo Rocket

Load resources into a cargo rocket silo and configure it to send to a cargo landing pad.
Cargo rockets have 500 slots.
Only suitable for high-bandwidth items. Won't automatically launch if the destination pad is full.  

## Mixed Cargo Rocket

The most complex option but is the best option for space logistics.

### Signaling Protocol

The basic idea is that the outpost sends two signals: (red) the total number of items it wants to have in
its storage buffer, and (green) the items it currently has in its storage buffer.

This signal is sent on a Signal transmitter (from AAI Signal Transmission) to a Signal reciever on a
supply area, on a channel named `<name of surface> Requests`. Additionally, the red channel
should be negated (i.e. a request of 500 iron plates should be denoted with -500 iron plates).

> Warning: Be sure to capitalize (with title case) the channel name! Channels cannot be renamed afterwards.

### Pitfalls

We start with a naive implementation that subtracts inventory (adding inventory from the current cargo contents)
from requests and uses filter inserters to insert items into the cargo that are below requests.

#### Not enough requests

The naive implementation will simply launch the rocket when it is full. However, if one resource
is completely empty, then the outpost might need to wait on that item before it is able to do any more work and
consume the other resources. The problem occurs if the requests do not add up to 500 stacks, in that case the rocket will never be sent
because the sender is waiting on the requester to request more stuff, and the requester is waiting on the
sender to send more stuff.

The problem can be more easily triggered if the rocket loading only loads the first few items in the request until they are full before
moving to other item types. This unbalances the item ratios on the outpost, leading to situations where only 1-2 item types are is missing,
making it very easy for the first problem to occur. 

The solution to this problem is to use a better loading strategy, many of which will be covered below.

#### Transiently forgetting about the sent cargo

When the cargo rocket is launched, the wire tracking the cargo contents will no longer report any items, since the
rocket is gone. However, this means that the outpost will continue requesting the items that were just sent, since
the cargo has not been recieved yet. This means the loader will load the same items that were just sent.

The solution is to create a lockout time, when the loader is disabled for ~24 seconds after the rocket is launched.

### Loading Strategies

There are a few different loading strategies, however the one we are using is _enhanced naiveloading_.

#### Naive-loading
- subtract demand from cargo contents
- set filters to signals that are negative
problems:
- unless request sizes are all bigger than 1 cargo rocket, can get deadlocked
- can fix deadlock by not filling rockets all the way
#### Enhanced Naive-loading
- subtract demand from cargo contents
- set filters to signals that are negative
- if the outpost reports a significant resource pressure, then begin normloading the rocket to fill it
problems:
- either needs extra logic on the outpost, or needs 2 wires to send back
#### Norm-loading
- scale the demand by a coefficient, A
- adjust A so that the total demand fills a cargo rocket
- subtract scaled demand from cargo contents
- set filters to signals that are negative
problems:
- if the outpost is full on most item types but is low on 1, can cause a oversupply of that item
#### Max-loading
- subtract demand from cargo contents
- find the most negative signal and set filters to that
problems:
- still suffers from the deadlock problem, but is marginally better at recovering from a high demand situation than naiveloading
- high complexity