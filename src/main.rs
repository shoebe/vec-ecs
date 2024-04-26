pub mod comp_iter;
pub mod comp_vec;
pub mod entity_handle;

use comp_iter::*;
use comp_vec::*;
use entity_handle::*;

#[derive(Debug)]
pub struct Position(f32, f32);

#[derive(Debug)]
pub struct Velocity(f32, f32);

pub struct Entity {
    pub position: Position,
    pub velocity: Velocity,
}

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

/*
Goal:

pub struct World {
    position: Collec<Position>,
    veclocity: Collec<Velocity>,
}

world.insert(
    Entity {
        position: Position(1.0, 1.0),
        velocity: Veclotiy(1.0, 1.0),
    }.bundles()
);

let entity = world.get::<Entity::Borrow>(); -> does this take &mut world??

let vec: &[Position] = world.get_column::<Position>();

*/

fn main() {
    println!("Hello, world!");
}

use vec_ecs_macro::comp_iter;

#[test]
fn test() {
    let mut world = World::default();
    let e = world.new_entity();
    world.pos.insert(e, Position(0.0, 0.0));
    world.vel.insert(e, Velocity(10.0, 0.0));

    comp_iter!(&mut world.pos, &mut world.vel; |id, pos, vel| {
        dbg!(id, pos, vel);
    });
}
