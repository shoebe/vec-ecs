mod comp_iter;
mod comp_vec;
mod entity_handle;

pub use comp_iter::*;
pub use comp_vec::*;
pub use entity_handle::*;
pub use vec_ecs_macro::*;

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
