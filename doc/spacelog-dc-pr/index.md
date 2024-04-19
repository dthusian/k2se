# Space Logistic Proposal: Delivery Cannon Only
## Project Overview
This proposal covers the most naive and simple approach to space logistics, which is to use the delivery cannon for all materials sent in space. It also attempts to cover all the potential downsides and difficulties of using delivery capsules while also pointing out areas where it excels compared to the other proposals (cargo rocket). 

## Technology Introduction
DCs use energy to shoot capsules with items to DRs on other planets or orbit. It requires one capsule per shot, can only carry certain items as payload (often raw resources) and carry exactly 1 stack of items per capsule.

DC = Delivery Cannon
DR = Delivery Reciever (chests)

## Proposal Goals
This proposal is built around two main goals:
#### Simplicity
The main advantage of the DC approach to space logistics is simplicity and in turn being completely idiot-proof. This approach allows for the fewest possible points of failures. Logistic deadlocks should almost never be a fault of the DCs, and even if they are, they should be very easy to resolve.  

#### Ease of use
The DC system should be easy to use. Documentation should be brief and intuitive while implementation should be as easy on the user as possible. 

## Logistics system overview
### High level outline
The idea at the core of the DC system is that only raw materials shall be delivered, while assembly of the final product shall be done on site. A DC system's delivered materials will consist solely of the most dense options in order to save costs on capsules (such as ingot varieties or ice varieties). Once these resources are delivered to the destination's DR, assembly of the needed resource will take place on site.

### Delivered Items Selection
As hinted at before, consideration must be put into what items should be put on the DC system, similar to how one would consider what to put on a main bus or cityblock system. Items on the system should adhere to at least one of the following conditions:
1. This item is currently unobtainable on the DC system (and some planet needs it)
2. This item is unreasonably difficult to create with the DC system, and is not needed in large amounts
3. This item is commonly used and dense, even if it is already possible to create from items on the DC system

Some basic materials that will almost definitely be on the DC system are: 
> Any variety of Ingot
> 
> Water ice
> 
> Uranium 235 and 238
> 
> LDS (due to reason 2)

One exception will be the 4 final intermediate ingredients to craft a capsule, which will be covered in a later section.

### Requester Stations
Requesting stations refers to DRs placed on planets. These are modular and can be placed onsite. One requester station is used to source materials to produce one item as a goal (such as UFCs, uranium text plates, or meteor defense ammo). These requester stations can should always be blueprinted for ease of use and consists of the following parts:
1. A single DR
There should only be a single DR per station and all raw materials should be sent here. Filter stack inserters should be used to sort the items into storage chests/warehouses. 
2. A radar station + constant combinator
This should be used to report current inventory and the amount needed to the supply station. The exact documentation for how this works will be explained in a later section. Under the current proposal each requesting station will have its own channel on the signal transmitter. 
3. Assembling stations
A line of stuff that manufactures the raw materials to the end result. 

One requester station should have the capacity to make a reasonable amount of the end material. Whether this is needed or not can be handled by the network system. 

The supplying to the requester station must also be configured from a supplier station as well. 

##### Examples:
Green circuits (nothing is ratioed and doesn't produce very much but you get the idea)
![enter image description here](https://i.imgur.com/fv0ZheU.png)

UFCs

![enter image description here](https://i.imgur.com/l4cOiTz.png)
##### potential
Maybe add something for a hub requester station that can produce something like a mainbus of raw materials if needed, like cargo rockets would be able to 

### Supplier Stations
It should be noted that the DC system does not have a LTN-like system, in that if you wanted to supply a requester station, the system will not automatically assign a new supplier station for it and the user must assign one themselves. This is also a issue with the CR system i think (cargo rocket destinations cannot be automatically assigned)

One supply station supplies a single resource to the DC system. The supply station will usually produce a large amount of one material and supply it to requester stations. 

Supply stations will also need to import materials to create delivery capsules. They can import them directly from capsule supplier stations, outlined below. 

For simplicity, it is important to note that one requester station should only source raw material from one supplier station per resource, as to avoid situations where multiple supplier stations attempt to supply a single destination, overfilling it and causing damage. 

In addition, each planet should only have one supply station per material. If a planet is set as a copper outpost planet, it should congregate all copper ingots to one location to be sent to requester stations. In addition, this means that supplier stations are not as modular and should be made per planet. 

#### Supply Array
As mentioned before, each DC can only supply one requester station. As such, at each supplier station there will be an array of DCs and signal receivers. This should be easy to use as well, as the user will only need to set the channel of the signal receiver on one of the DC supply arrays to the one from the receiver station. Ideally there should be a line of unused DCs and signal receivers that can be set to receiver stations as needed.


### Special Supplier Station - Capsule Suppliers
Supplier Stations need a large amount of capsules to send. As such, we must have stations that are dedicated to creating delivery capsules. Planets will be designated as capsule suppliers and will either import or mine materials to manufacture the final intermediate ingredients for supply capsules - LDS, explosives and heat shielding. Copper cables can be created from imported copper ingots. All 3 of these intermediates can be shipped, and so should be supplied via DCs to supplier stations. 

### Special Supplier Station - Planet Trash
TODO (idk if this is even a good idea)

### DC Network Documentation
TODO (just use the preexisting protocol, also make a naming scheme)

## User Implementation
In short, this is the procedure to create a supply of any non-raw item on any planet:
1. Paste down (or create) the blueprint for the receiver station that produces this item
2. Set up a channel on the signal transmitter
3. Go to the corresponding supplier stations for each raw resource that has to be imported and set a DC on the supply array

## Cost Benefit Analysis
The DC space logistic proposal will be compared to the CR proposal, as that is our only alternative right now

#### Benefits
1. Power efficiency and stability

>DCs uses 4x less power than cargo rockets per stack of items to be shipped ([citation](https://docs.google.com/spreadsheets/d/1jcZrj4KvTgKQEUVb6SOjYYj1NO_6mJ86vSWs2DHxeF0/edit#gid=0)). This is important on waterless planets and can result in a more stable electricity supply which leads to not needing steam batteries to deal with CMEs. 
2. Electricity as fuel
   
>Does not need rocket fuel or anything, and is therefore easily implemented on any planet. Planets without water must import in water ice or core mine for rocket fuel, both of which are not really ideal (1 cargo rocket to nauvis orbit = 40 capsules of water ice + needs a lot of electricity to treat and turn into rocket fuel, and pyroflux from core mining comes in small quantities and produces a lot of byproducts).
3. Significantly simpler logistics
>The DC system has pretty much no chance to deadlock. Any issues that cause a failure in the DC system can be easily fixed and would usually be because some resource ran out, not because the system was too complicated and failed in a unforeseeable way. There is no need for rocket cargo parts recycling or buffering rocket sections to prevent a surplus. 
In addition, there is no need for specialized algorithms like enhanced naive-loading. There is also no consideration needed for planets who do not send any rockets back, such as science stations, and so will need to get rid of rocket parts. All of this leads to fewer potential points of failure and easier usage. 
4. Easier and faster implementation
>Whereas CR systems need a central hub for all resources to be pooled, the nature of receiver stations allows for intermediate or final items to be sourced onsite. The way that receiver stations are all blueprints also allows for less thinking to be involved and allows for them to be placed onsite for anything that's needed. The 3 step process to implementing a requester station as outlined above is very easy. 
#### Pitfalls
1. Less cool
>self explanatory. bottom text
2. Channel bloat
>Because each requester station uses its own channel, it can easily lead to a large number of channels being in use. This leads to DCs potentially not being viable in the long term
3. Materials needed to run
>While CRs can be researched to use almost no materials to sustain, DCs will always need a linear amount of not cheap resources to produce capsules which can add to difficulty of logistics in the long term. 
4. Range
>CRs have more range than DCs and DCs cannot go outside the star system. While this could be remedied in the future with a hub and spoke system where we CR items to other star systems and have a localized per-planet DC network, this is still a downside for now. 
