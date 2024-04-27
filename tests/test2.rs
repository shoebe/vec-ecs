use vec_ecs::{CompIter, CompVec, EntityHandleCounter, World, WorldBorrow};
#[derive(Debug)]
pub struct Position(f32, f32);

#[derive(Debug)]
pub struct Velocity(f32, f32);

#[derive(Debug)]
pub struct Flag(bool);

#[test]
fn test_derive() {
    #[derive(vec_ecs::World, Default)]
    pub struct World {
        #[world(handles)]
        handles: EntityHandleCounter,
        #[world(struct_borrow_without)]
        pub pos: CompVec<Position>,
        pub vel: CompVec<Velocity>,
        pub flags: CompVec<Flag>,
    }

    #[derive(vec_ecs::Entity, Debug)]
    #[entity(insert = World)]
    #[entity(borrow = WorldNoPos)]
    pub struct Entity {
        vel: Velocity,
        flags: Flag,
    }

    let mut world = World::default();

    let e = Entity {
        vel: Velocity(10.0, 10.0),
        flags: Flag(true),
    };
    world.insert(e);

    let e = Entity {
        vel: Velocity(10.0, 0.0),
        flags: Flag(false),
    };
    let handle = world.insert(e);

    let (pos, mut world_no_pos) = WorldNoPos::split_world(&mut world);

    let e_borr: EntityBorrow = world_no_pos.borrow_entity(handle);
    dbg!(&e_borr);

    for (id, pos) in pos.iter_mut() {
        dbg!((id, pos));
    }

    for (id, vel, flag) in CompIter::from((world_no_pos.vel.iter_mut(), world_no_pos.flags.iter()))
    {
        dbg!((id, vel, flag));
    }
}
