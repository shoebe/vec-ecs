#[test]
#[allow(dead_code)]
fn test_derive() {
    use vec_ecs::{CompIter, CompVec, EntityBorrowTrait, EntityHandleCounter, WorldTrait};
    #[derive(Debug)]
    pub struct Position(f32, f32);

    #[derive(Debug)]
    pub struct Velocity(f32, f32);

    #[derive(Debug)]
    pub struct Flag(bool);

    #[derive(vec_ecs::World, Default)]
    #[world(borrow = WorldNoPos)]
    pub struct World {
        #[world(handles)]
        handles: EntityHandleCounter,
        #[world(not_in = WorldNoPos)]
        pub pos: CompVec<Position>,
        pub vel: CompVec<Velocity>,
        pub flags: CompVec<Flag>,
    }

    #[derive(vec_ecs::Entity, Debug)]
    #[entity(insert = World)]
    #[entity(borrow = WorldNoPos)]
    pub struct Player {
        vel: Velocity,
        flags: Flag,
    }

    let mut world = World::default();

    let e = Player {
        vel: Velocity(10.0, 10.0),
        flags: Flag(true),
    };
    world.insert(e);

    let e = Player {
        vel: Velocity(10.0, 0.0),
        flags: Flag(false),
    };
    let handle = world.insert(e);

    let (pos, mut world_no_pos) = world.split_world_no_pos();

    let mut e_borr = PlayerBorrow::from_world(handle, &mut world_no_pos);
    dbg!(e_borr.flags());
    dbg!(e_borr.vel_mut());

    for (id, pos) in pos.iter_mut() {
        dbg!((id, pos));
    }

    for (id, vel, flag) in CompIter::from((world_no_pos.vel.iter_mut(), world_no_pos.flags.iter()))
    {
        dbg!((id, vel, flag));
    }

    let _e_borr = PlayerBorrow::from_world(handle, &mut world);
}
