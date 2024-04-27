use vec_ecs::{CompIter, CompVec, EntityHandle, EntityHandleCounter};

#[derive(Debug)]
pub struct Position(f32, f32);

#[derive(Debug)]
pub struct Velocity(f32, f32);

#[derive(Default)]
pub struct World {
    handles: EntityHandleCounter,
    pub pos: CompVec<Position>,
    pub vel: CompVec<Velocity>,
    pub yomama: CompVec<()>,
    pub excluded: CompVec<()>,
}

impl World {
    pub fn new_entity(&mut self) -> EntityHandle {
        self.handles.next_handle()
    }
    pub fn delete_entity(&mut self, entity: EntityHandle) {
        self.handles.entity_deleted();
        self.pos.remove(entity);
        self.vel.remove(entity);
    }
}

#[test]
fn test() {
    let mut world = World::default();
    {
        let e = world.new_entity();
        world.pos.insert(e, Position(0.0, 0.0));
        world.vel.insert(e, Velocity(10.0, 0.0));
        world.yomama.insert(e, ());
    }

    {
        let e = world.new_entity();
        world.pos.insert(e, Position(2.0, 0.0));
        world.vel.insert(e, Velocity(10.0, 0.0));
    }

    {
        let e = world.new_entity();
        world.pos.insert(e, Position(3.0, 0.0));
        world.vel.insert(e, Velocity(10.0, 0.0));
        world.yomama.insert(e, ());
    }

    {
        let e = world.new_entity();
        world.pos.insert(e, Position(3.0, 0.0));
        world.vel.insert(e, Velocity(10.0, 0.0));
        world.yomama.insert(e, ());
        world.excluded.insert(e, ());
    }

    {
        let e = world.new_entity();
        world.pos.insert(e, Position(3.0, 0.0));
        world.vel.insert(e, Velocity(10.0, 0.0));
        world.yomama.insert(e, ());
    }

    for (id, pos, vel, yomama) in CompIter::from((
        world.pos.iter_mut(),
        world.vel.iter_mut(),
        world.yomama.iter().optional(),
    ))
    .without(&world.excluded)
    {
        dbg!((id, pos, vel, yomama));
    }
}

