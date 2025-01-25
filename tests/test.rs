#[test]
#[allow(dead_code)]
fn test() {
    use vec_ecs::{CompIter, CompVec, EntityHandleCounter, WorldBorrowTrait, WorldTrait};

    #[derive(Debug, Default)]
    pub struct Position(f32, f32);

    #[derive(Debug, Default)]
    pub struct Velocity(f32, f32);

    #[derive(vec_ecs::World, Default)]
    #[world(borrow = WorldNoNothing)]
    pub struct World {
        #[world(handles)]
        handles: EntityHandleCounter,
        pub pos: CompVec<Position>,
        pub vel: CompVec<Velocity>,
        #[world(not_in = WorldNoNothing)]
        pub nothing: CompVec<()>,
        pub excluded: CompVec<()>,
    }

    let mut world = World::default();
    assert!(world.is_empty());
    {
        let e = world.new_entity();
        world.pos.insert(e, Position(0.0, 0.0));
        world.vel.insert(e, Velocity(10.0, 0.0));
        world.nothing.insert(e, ());
    }
    assert!(!world.is_empty());

    {
        let e = world.new_entity();
        world.pos.insert(e, Position(2.0, 0.0));
        world.vel.insert(e, Velocity(10.0, 0.0));
    }

    {
        let e = world.new_entity();
        world.pos.insert(e, Position(3.0, 0.0));
        world.vel.insert(e, Velocity(10.0, 0.0));
        world.nothing.insert(e, ());
    }

    {
        let e = world.new_entity();
        world.pos.insert(e, Position(3.0, 0.0));
        world.vel.insert(e, Velocity(10.0, 0.0));
        world.nothing.insert(e, ());
        world.excluded.insert(e, ());
    }

    {
        let e = world.new_entity();
        world.pos.insert(e, Position(3.0, 0.0));
        world.vel.insert(e, Velocity(10.0, 0.0));
        world.nothing.insert(e, ());
    }

    for (id, pos, vel, nothing) in CompIter::from((
        world.pos.iter_mut(),
        world.vel.iter(),
        world.nothing.iter_mut().optional(),
    ))
    .without(&world.excluded)
    {
        dbg!((id, pos, vel, nothing));
    }

    let (nothing, mut world_no_nothing) = world.split_world_no_nothing();

    for (id, nothing) in nothing.iter_mut() {
        dbg!(id, nothing);

        for (id2, pos, vel, excluded) in CompIter::from((
            world_no_nothing.pos.iter(),
            world_no_nothing.vel.iter_mut(),
            world_no_nothing.excluded.iter().optional(),
        )) {
            dbg!((id2, pos, vel, excluded));
        }

        // make a new uninitialized entity while having part of world borrowed
        let handle = world_no_nothing.new_entity();
        dbg!(handle);
    }
}
