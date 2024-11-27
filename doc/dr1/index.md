# DR1: Space Logistic System Generation 2

Draft Revision 1a

## Miscellaneous Definitions

A Factorio item tag is a string of the form `[item=<item-id>]`, where `<item-id>` is some
Factorio item ID. In Factorio, these item tags are displayed as item icons in text fields.

## Surfaces

A Factorio world is composed of many surfaces. In Space Exploration,
each planet, orbit, or asteroid field gets one surface.

The term surface is from the internal Factorio name for them,
while SE also refers to surfaces as zones. This document will refer to surfaces as surfaces.

## Surface Types

## Transport Channels

All transport between different surfaces is done through transport channels. This
document recognizes 3 types of transport channels:
- Resource Channels
- Maintenance Channels
- Trash Channels

Each transport channel has at least one sender and at least one reciever. As their
name suggests, senders send items to recievers.

## Resource Channels

Resource channels are implemented with single-item many-to-many cargo rockets.
That is, any number of senders will be able to send to any number of recievers.

### Interface

Senders are cargo rocket silos configured as follows:
- Destination set to "Any Landing Pad With Name"
- Landing pad set to `g2::resource <item>`, where `<item>` is a Factorio item tag
- Launch set to "Launch when Cargo full"

Recievers are landing pads named `g2::resource <item>`, where `<item>` is a Factorio item tag.

Since senders rely on destination pads being full in order to select landing pads,
recievers should not create large secondary buffers, instead, buffering items on the landing pad itself. 

### Low Priority Senders

It is sometimes desirable for a sender to have a lower send priority. For example, cargo
rocket sections and space capsules are reusable and thus are often sent between surfaces.
If there are sufficient rocket sections in circulation, then the surface which produces
cargo sections should not export more cargo sections into the system in case outposts become overfilled.

`todo!()`

## Maintenance Channels

Maintenance channels are implemented with mixed-item point-to-point cargo rockets.
Due to the point-to-point nature, each reciever must have a sender configured to send to it.

### Interface

On the reciever side, a signal transmitter sends negative item requests on the red
wire and positive inventory levels on the channel `g2::maintenance <surface>`,
where `<surface>` is the case-sensitive name of the surface.

The sender then reads the transmitted signals and sends corresponding items to the
reciever. While many strategies can be used to load mixed rockets, implementations must
use "for-loading" as it is the most robust known loading strategy.

#### For-loading Specification

The loader operates on a control loop. The behavior of the loader described below.

Every loop iteration:
- Read the config combinators into `warnThres`, `lowThres`, `highThres`
- Read the current requests into `req` and inventory into `inv`.
  - Requests are negative and inventory positive like discussed above.
- Let `a` = 100, memory cell which persists across cycles
- Let `ratio` = `inv` / `req`
- Let `scaledRatio` = `inv` / (`req` * `a` / 100)
- Let `filt` = Only items in `scaledRatio` that are between `lowThres`% and `highThres`%
- If any item in `ratio` is below `warnThres` AND `filt` is empty:
  - Add 5 to `a`  
- Otherwise
  - Set `a` to 100
- Set inserter filters to items in `filt`

## Trash Channels

Trash channels are implemented with mixed-item point-to-point cargo rockets.
All trash channels are sent to one surface designated as the global trash handler.
However, since the specific items on the rocket itself are not significant, no complex
logic is required.

### Interface

Senders are cargo rocket silos configured as follows:
- Destination set to "Any Landing Pad With Name"
- Landing pad set to `g2::trash`
- Launch set to "Launch on green signal"

A circuit condition should be configured to launch the rocket when there are no more
empty slots.

The reciever side is a landing pad named `g2::trash`.

## Surface Management

`todo!()`