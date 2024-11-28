# RFC 7+2i: Move-assignment Train System for Gleba

> Draft 3

## 0. Terminology

- _Italics_ will be used when introducing a new term.
- _Fruit_ refers to either Yumako or Jellynut items.
- _Processed fruit_ refers to either Yumako Mash or Jelly items.
- _Seed_ refers to Yumako seed or Jellynut seed items.
- _Plantation_ refers to a production area that grows seeds into fruit.

## 1. Motivation

Gleba presents a completely new approach to Factorio logistics. Due to spoilage, the common
practice of using backpressure to distribute items evenly will not work. This also means
that 1.1 many-to-many train systems cannot work effectively either.

This document presents a new system for train logistics with spoilable items, intended for use with Yumako or Jellynut fruit.

## 2. High-level Overview

Even among spoilable items, fruits have unique logistical challenges in that seeds are needed to produce more
fruit. Seeds can only be obtained by processing fruit, but processed fruit spoils quickly and as such is impractical
to transport. This means that fruit processing cannot be done at the plantations, and seeds must be returned separately.

The _move-assignment train system_ (MATS) is designed to effectively address this challenge.

_Fig 2.1: MATS Block Diagram_
```
+------------+ <-----------------+
| Plantation | <------------+    |
+------------+              |    |
      |                     |    |
      | Train               |    |
      | (fruits)            |    |
      V                     |    |
+------------+              |    |
| Processing |              |    | 
|    Area    |              |    |
+------------+              |    |
      |                     |    |
      | Train               |    |
      | (leftovers)         |    |
      V                     |    |
+------------+              |    |
| Leftovers  | -------------+    |
|  Handling  |     Seeds         |
+------------+                   |
      |                          |
      | Train                    |
      | (empty)                  |
      V                          |
+------------+                   |
|   Train    | ------------------+
|   Buffer   |    Train (empty)
+------------+
```

Trains start at one of many plantations (fruit provider), which fills them with fruits.
Once full, they are routed to one of many _processing areas_ (fruit requester), which uses the fruits for production.
While in a standard requester train stop, the train leaves when it is empty,
in a MATS requester train stop, the train leaves when another train is waiting in the station.
Since the train that left may still have fruits inside of it, it is sent to a
_leftovers handling area_, where the remaining fruits are processed for seeds and processed fruits
discarded. The empty train now heads to a _train buffer_ to wait until it can go to the plantation again.

The action where a train arrives at a requester train stop, removes the existing train, and moves
itself into the stop is similar to the C++ move-assignment constructor, hence the name move-assignment train
system.

### 2.1. Advantages and Disadvantages

Advantages of MATS include:
- No fruit is needlessly discarded. That is, there will never exist a situation where some processing areas
  are starved for fruits but fruit is being discarded.
- Scalable. MATS supports an arbitrary number of plantations and processing areas.
- Optimized for freshness.
- Excess fruits can be collected in a central area and used for power generation.

Disadvantages of MATS include:
- No spore management. Plantations can always be active, producing spores that induce attacks on the base.
- Request depth is limited to 2, since requesting more trains will simply cause all but the last to be removed immediately.

## 3. Functional Description

### 3.1. Plantation

The plantation receives seeds from the seed requester and grows them into fruits with agricultural towers.
This process is always active, because trees have a lead time of 5 minutes, it is difficult to throttle agricultural
towers effectively.

Fruits are loaded into trains until they are full and then sent to processing areas. The plantation's train
stop should have a large request depth.

### 3.2. Processing Area

A processing area is any area that uses fruits to create other products.

Fruits are requested via a fruit requester station. A processing area might have other stations that are
outside the scope of this document.

Fruit requester have a train limit of 2. Normally, one train is parked in the station. If another train arrives,
the first train is removed. This is done by reading the train signal before the 2nd train slot, and 
sending that circuit signal to the train, which is configured to leave when the train signal is red.

Fruit requesters have their priority set to `(T - x) / (T / 100)`, where `x` is the amount of fruits
inside the cargo of the current train, `T` is the number of fruits fit inside of a train.
This causes trains to be distributed to the most empty requester, which balances the distribution of trains
across all requesters. 

Fruit requesters must not unload trains into chests. The intent is that the train itself is used as a buffer to pull from.
The advantage of using a train for it is that the train can be instantly flushed so that only the freshest products will
be used.

Some processing areas may implement a shutdown mechanism. If the processing area shuts down, the fruit requester
must set its train limit to 0, and optionally may remove currently parked trains.

### 3.3. Leftovers Handling

The leftovers handling area fully cleans out trains that have been removed from a fruit requester.
This is required because trains can be removed from the fruit requester without being empty.

The leftovers handling area should unload trains into chests for faster unloading.
Fruits are processed into seeds and processed fruit. Seeds from processing and seeds from the train are then sent to
the plantations. The processed fruit can be used for power generation or discarded.

### 3.4. Train Buffer

The train buffer area consists of a waiting area where trains can wait for an open slot in the fruit provider.

### 3.5 Seed Transport

Seeds must be transported from the leftovers handling area to the plantations. Since seeds do not spoil,
and seeds are used in much smaller quantities this can be accomplished using conventional trains or logistic bots.