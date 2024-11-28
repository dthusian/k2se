# RFC 7+2i: Move-assignment Train System for Gleba

> Draft 1

## 0. Terminology

- _Italics_ will be used when introducing a new term.
- _Fruit_ refers to either Yumako or Jellynut items.
- _Processed fruit_ refers to either Yumako Mash or Jelly items.
- _Seed_ refers to Yumako seed or Jellynut seed items.
- _Plantation_ refers to a production area that grows seeds into fruit.

## 1. Motivation

Gleba presents a completely new approach to Factorio logistics. Due to spoilage, the common
practice of using backpressure to distribute items evenly will not work. This also means
that vanilla many-to-many train systems cannot work effectively either.

This document presents a new method for train logistics with spoilable items, intended for use with Yumako or Jellynut fruit.

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

Trains are routed to the fruit requester with the highest priority, which is set to be higher the less fruits
are in the currently buffered train. This balances trains to service the most demanding processing
areas without neglecting less demanding areas.

Seeds are sent from the leftovers handling area back to the plantations in an implementation-defined way.
For low throughput, logistic robots can be used, or alternatively a standard many-to-many train system can be used.

### Advantages and Disadvantages

Advantages of MATS include:
- No fruit is needlessly discarded. That is, there will never exist a situation where some processing areas
  are starved for materials but some fruit is being discarded.
- Scalable. MATS supports an arbitrary number of plantations and processing areas.
- Excess fruits can be collected in a central area and used for power generation.

Disadvantages of MATS include:
- No spore management. Plantations can always be active, producing spores that induce attacks on the base.
- Request depth is limited to 2, since requesting more trains will simply cause all but the last to be removed immediately.

## 3. Functional Specification

### 3.1. Plantation

The plantation contains an implementation-defined seed requester, and one or more fruit providers.

- (1) Each fruit provider is a train stop.
- (2) Fruit providers must load the train with fruits, and remove the train when it is full.
- (3) To deal with possible spoilage, fruit providers must also remove the train if fruit cannot be inserted into the train.
- (4) Fruit providers should request multiple trains to maximize throughput.

- (5) The seed requester should transport seeds from the leftovers handling to the plantation.
- (6) The seed requester should not buffer unlimited amounts of seeds.

### 3.2. Processing Area

The processing area contains one or more fruit requesters.

- (1) Each fruit requester is a train stop with 2 train limit.
- (2) Fruit requesters must remove the current train if more than 1 train is inside the requester.
- (3) Fruit requesters must set their priority according to the expression `(T - x) / (T / 100)`, where
  `x` is the amount of fruits inside the cargo of the current train, `T` is the number of fruits fit inside of a train.
- (4) The seeds produced from processing fruits must be loaded back onto the train in the corresponding fruit requester.
- (5) Fruit requesters must not unload into chests.

- (6) Processing areas may implement a shutdown mechanism.
      During shutdown, the fruit requester should remove all trains and must set its train limit to 0.

### 3.3. Leftovers Handling

The leftovers handling area contains one or more leftover requesters.

- (1) Each leftover requester is a train stop.
- (2) Leftover requesters should request multiple trains to maximize throughput.
- (3) Leftover requesters should unload into chests to reduce the time a train spends inside of it.

- (4) Leftovers handling must unload fruits and seeds from trains and processes them.
- (5) The processed fruits must be discarded in an implementation-defined way.
- (6) The seeds must be sent to plantations in an implementation-defined way, or may be discarded if there is an excess.

### 3.4. Train Buffer

The train buffer area consists of a waiting area where trains can wait for an open slot in the fruit provider.