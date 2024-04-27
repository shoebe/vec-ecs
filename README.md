# vec-ecs
An ECS with vector and bitset backed components

## Goals
* Easy to use ECS for small numbers of entities

## Non-goals
* speed
* change detection
* [hecs](https://github.com/Ralith/hecs) style dynamic borrow checking
* [bevy](https://github.com/bevyengine/bevy) style automatic parallel system scheduling


## Features
### Ability to split the world into a component vec and the other component vecs
```rust
#[derive(vec_ecs::World, Default)]
pub struct World {
    #[world(handles)]
    handles: EntityHandleCounter,
    #[world(split_off = WorldNoPos)]
    pub pos: CompVec<Position>,
    pub vel: CompVec<Velocity>,
    pub flags: CompVec<Flag>,
}
```
The `#[world(split_off = WorldNoPos)]` label generates the struct:
```rust
pub struct WorldNoPos<'a> {
    pub vel: &'a mut CompVec<Velocity>,
    pub flags: &'a mut CompVec<Flag>,
}
```
And allows:
```rust
let mut world = World::default();
// Add entities
// ...
let (pos, mut rest) = world.split_pos();
// pos: &'a mut CompVec<Position
// rest: WorldNoPos<'a>
```

### Component iteration
```rust
#[derive(vec_ecs::World, Default)]
pub struct World {
    #[world(handles)]
    handles: EntityHandleCounter,
    pub pos: CompVec<Position>,
    pub vel: CompVec<Velocity>,
    #[world(split_off = WorldNoNothing)]
    pub nothing: CompVec<()>,
    pub excluded: CompVec<()>,
}
// init world, add entities
// ...
for (id, pos, vel, nothing) in CompIter::from((
    world.pos.iter_mut(),
    world.vel.iter(),
    world.nothing.iter_mut().optional(),
)).without(&world.excluded)
{
    // id: EntityHandle
    // pos: &mut Position
    // vel: &Position
    // nothing: Option<&mut ()>
    // will skip any entities with the `excluded` component
}

let (nothing, world_no_nothing) = world.split_nothing();

for (id, nothing) in nothing.iter_mut() {
    // id: EntityHandle
    // nothing: &mut ()
    for (id2, pos, vel, excluded) in CompIter::from((
        world_no_nothing.pos.iter(),
        world_no_nothing.vel.iter_mut(),
        world_no_nothing.excluded.iter().optional(),
    ))
    {
        // id2: EntityHandle
        // pos: &Position
        // vel: &mut Position
        // excluded: Option<&()>
    }
}
```

### Entity insertion/borrows
```rust
#[derive(vec_ecs::Entity)]
#[entity(insert = World)]
#[entity(borrow = WorldNoPos)]
pub struct Player {
    vel: Velocity,
    flags: Flag,
}
```
The `#[entity(insert = World)]` allows:
```rust
let e = Player {
    vel: Velocity(10.0, 10.0),
    flags: Flag(true),
};
let handle = world.insert(e);
```

The `#[derive(vec_ecs::Entity)]` also generates the following "borrow" struct:
```rust
pub struct PlayerBorrow<'a> {
    vel: &'a mut Velocity,
    flags: &'a mut Flag,
}
```
And the `#[entity(borrow = WorldNoPos)]` allows
```rust
let e_borr: PlayerBorrow = world.borrow_entity(handle);
// and
let (pos, mut world_no_pos) = world.split_pos();
// pos: &'a mut CompVec<Position
// world_no_pos: WorldNoPos<'a>
let e_borr: PlayerBorrow = world_no_pos.borrow_entity(handle);
```

Note: all the fields in structs labeled with `#[derive(vec_ecs::Entity)]` must have the same names and types as the fields in the struct in the `#[entity(insert = ...)]` label and the structs in the `#[entity(borrow = ...)]` labels

## TODO:
Needs more testing for entity insertion/removal and iterating

