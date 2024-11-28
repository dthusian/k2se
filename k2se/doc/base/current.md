# Current State of Affairs

This document documents the current active bases on all surfaces.

## Nauvis (old base)

> Status: Abandoned
> Power System: Heisenberg

Contains Basic/R/G/B science and mall.

## Nauvis (bus base)

> Status: Deprecated
> Power System: Heisenberg

Contains rocket silo that supplies Nauvis orbit, and delivery cannons that supply multiple outposts.

### Connections

Send (cargo rocket) to Nauvis Orbit:
- See config combinators on Nauvis Orbit

Send (delivery cannon) to Miochin:
- LDS
- Explosives
- Heat shielding
- Sulfur

Send (delivery cannon) to Rhadaman:
- Steel

Recv (delivery cannon) from Rhadaman:
- Imersite (temporary)

## Nauvis (city block)

> Status: Under Construction
> Power System: Heisenberg

Mostly finished, see Jira for exactly what is missing.

### Connections

Recv (delivery cannon) from Nauvis (bus base):
- Vulcanite

## Nauvis Orbit

> Status: Active

Used to make space sciences.

### Connections

Recv (cargo rocket) from Nauvis (bus base):
- See config combinators on Nauvis Orbit

## Miochin

Status: Active
Power System: Nuclear

Primarily used for vulcanite extraction.

Recv (delivery cannon) from Nauvis (bus base):
- LDS
- Explosives
- Heat shielding
- Sulfur

Recv (delivery cannon) from Rhadaman:
- U-235
- U-238

Send (delivery cannon) to Nauvis (city block):
- Vulcanite

## Rhadaman

Status: Active
Power System: Nuclear

Peter set this outpost up when the first uranium patch ran out on Nauvis.
Constructs delivery cannon materials on-site

Recv (delivery cannon) from Nauvis (bus base):
- Steel

Send (delivery cannon) to Miochin:
- U-235
- U-238

Send (delivery cannon) to Nauvis (bus base):
- Imersite (temporary)

## Minos

Status: Abandoned

Temporary outpost to get enough cryonite for logistic network research.