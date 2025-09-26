# Survicraft

A simple multiplayer game using Bevy.

## Quickstart

### GUI Application

```console
cargo run --bin survicraft
```

### Dedicated Server

```console
cargo run --bin survicraftd
```

### TODO

- [ ] implement a crafting system
    - [x] implement an example using the crafting system
    - [x] refactor the example into a plugin for the game
    - [x] items dropped on the ground have physics
    - [x] cast a spehere on the grouund where the player is looking at (we can have a component for the thing that casts the sphere)
    - [x] if the debug feature is enabled we display the crafting sphere for the player (it will be at the intersection with the ground)
    - [x] we identify the items inside of the sphere
    - [x] if the debug feature is enabled we display the items that are in the crafting sphere
    - [x] if we press `F` (the use key) we craft the item
    - [x] if an item matches a recipe we display that we can craft it
    - [ ] if we have multiple items we can cycle through them with scroll
    - [ ] display a nice UI (maybe above the chat for the game) with the current item we can craft
    - [ ] maybe somehow highlight the items that will be used
- [ ] implement a storage system
    - I want the storage system to not have a UI, but rather be more graphical, you should be able to have something like a chest,
    and you should place items inside, the placed items will not have physics, but then you should be able to take them and drop them
    to add physics to them another idea is to have a backpack on which you can attach items, and it should work kind of similar
- [ ] build workflows for different OS
    - How can I get wasm to work (for the client)
- [ ] better character controller
    - [x] try to use kinematic approach
- [ ] Markup language for assets (as described in the comments of the assets)
- [ ] Chat commands for "cheating" like spawning items or teleporting etc.
- [ ] somehow more modding support: I would like the core of the game to be a "mod" so that new things can get added more easily
