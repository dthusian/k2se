# Space Logistics Gen2 Whitepaper

## Problem

Currently, space logistics is primarily done by single-item delivery cannons with a
cargo rocket silo to Nauvis orbit for producing science packs. Additionally, Nauvis as a central cargo hub
will not scale to multiple planets and systems.
This paper will outline a new (dubbed "gen 2") system of delivery that covers all logistic workloads and
is more scalable with lower maintenance.

## Proposal

### Transport Channels

Items are delivered on well-defined transport channels. Each channel has different behavior and different uses.
A surface might have multiple transport channels.

This paper defines 3 different types of transport channels:
- Resource
- Maintenance
- Trash

### Resource Channels

Resource channels can only carry 1 item type, and are used for bulk single resources.
For example, an iron mining outpost might have a resource channel that carries iron ingots.
Resource channels are implemented with the simple cargo rocket loader, that is, loading
rockets until they have no more free space.

#### Topology

Resource channels use a many-to-many topology. Each cargo rocket silo has a destination configured to
"any landing pad with name", which will cause it to select the next free landing pad that requests a certain item type.
The advantage of this system is reduced maintenance and complexity cost compared to delivery cannons as individual
point-to-point delivery channels do not need to be maintained.

### Maintenance Channels

Maintenance channels can carry any number of item types, and are used for outpost maintenance.
This includes items like meteor defense ammo, uranium fuel cells, or repair packs.
Maintenance channels are implemented with the mixed-item cargo rocket loader.

#### Topology

Each maintenance channel connects to a local logistics hub that is usually in the same star system. That logistics hub
needs to manually configure each surface it is serving. Usually, one surface in each star system is designated as a logistics hub.
If needed, it is possible to service maintenance from a different star system at the cost of rocket fuel.

Logistics hubs should contain a mall that produces common items that outposts may use. A catalogue of these items should be
documented so that outposts are not stuck trying to obtain the logistics hub doesn't make.

### Trash Cannels

Trash channels can carry any number of item types and are used to carry waste items from an outpost.
This might include processing byproducts.
Trash channels are implemented with the simple cargo rocket loader.

#### Topology

Trash rockets are all sent to a universal trash handler. This was chosen because trash is a relatively low-bandwidth
channel, so the potential rocket fuel savings from having a multi-tiered trash handling system were outweighed
by its complexity.

The trash handler can hold onto the items, crush them, or attempt to recycle them. Any of these options is acceptable. 

### Rocket Parts Logistics

Since this system makes heavy use of cargo rockets, each surface needs to obtain large amounts of rocket fuel
and rocket sections in order to send rockets.

#### Rocket Sections

Packed cargo rocket sections are delivered in one of a few ways, determined by the rocket section surplus.
When a cargo rocket arrives at a surface, some of its sections will be reusable, and so each surface naturally
gets cargo sections. The rocket section surplus represents the net change in rocket sections on a surface when
factoring in sending and recieving rockets from all channels.

- Surfaces with large negative rocket section surplus get rocket sections through resource channels.
- Surfaces with small negative rocket section surplus get rocket sections through maintenance channels.
- Surfaces with positive rocket section surplus send rocket sections through resource channels.
- Surfaces with an ambiguous surplus should setup resource channels to send and recieve rocket sections, with
  circuit conditions to regulate them.

The distinction between large and small is subjective, and the specific solution used does not greatly matter.

#### Space Capsules

Space capsules are significantly more compact that cargo rocket sections, and are able to be supplied through
maintenance channels or dumped through trash channels.

#### Rocket Fuel

On planets with water, rocket fuel can be trivially created on-site with the ammonia recipe, albeit taking
lots of power. Iron can be supplied through maintenance.

Planets without water should recieve water ice through resource channels to create rocket fuel.

### What about delivery cannons?

Delivery cannons have had their time. Delivery cannons have the following problems:
- They are not scalable.
  Each delivery cannon has a bandwidth of at most about 7.5 stacks/m (50 MW * est. 400 MJ for planet->orbit),
  while cargo rockets can have a bandwidth of 250 stacks/m, and more if optimized.
- They much more expensive in terms of materials than beryllium cargo rocket sections.
- They cannot move items between star systems.
  This is significant because a delivery-cannon based setup would now need a complex
  delivery system to integrate with cargo rockets so that its resources can pass to other star systems.
- They have item restrictions, making them unsuitable for maintenace channels.
  Without maintenance channels, on-planet outposts need to become more complex to manufacture needed items onsite.

For these reasons, delivery cannons have no place in a modern space logistics system.