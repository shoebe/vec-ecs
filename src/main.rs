use fixedbitset::FixedBitSet;

mod lending;
pub use lending::*;

pub struct Position(f32, f32);
pub struct Velocity(f32, f32);

pub struct Entity {
    pub position: Position,
    pub velocity: Velocity,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EntityHandle {
    index: usize,
    gen: u32,
}

#[derive(Default)]
struct EntityHandleCounter {
    next: usize,
    gen: u32,
    removed: bool,
}

impl EntityHandleCounter {
    fn next(&mut self) -> EntityHandle {
        if self.removed {
            self.removed = false;
            self.gen += 1;
        }
        let n = EntityHandle {
            index: self.next,
            gen: self.gen,
        };
        self.next += 1;
        n
    }
    fn entity_deleted(&mut self) {
        self.removed = true
    }
}

pub struct ComponentStorage<T> {
    comps: Vec<(EntityHandle, T)>,
    // This guys never shrinks
    owners: FixedBitSet,
    // Used for lending iterator
    lending_removed_a_comp: bool,
}

impl<T> Default for ComponentStorage<T> {
    fn default() -> Self {
        Self {
            comps: Default::default(),
            owners: Default::default(),
            lending_removed_a_comp: Default::default(),
        }
    }
}

impl<T> ComponentStorage<T> {
    pub fn get(&self, id: EntityHandle) -> Option<&T> {
        if self.owners.contains(id.index) {
            let comp_ind = self.owners.count_ones(0..id.index); // exclude self
            let (id_out, comp) = &self.comps[comp_ind];
            if *id_out == id {
                return Some(comp);
            }
        }
        None
    }

    pub fn insert(&mut self, id: EntityHandle, comp: T) -> Option<T> {
        assert!(!self.lending_removed_a_comp);

        let already_had = self.owners.contains(id.index);

        self.owners.grow_and_insert(id.index);

        let comp_ind = self.owners.count_ones(0..id.index); // exclude self

        if already_had {
            let (old_id, comp) = std::mem::replace(&mut self.comps[comp_ind], (id, comp));
            assert_eq!(old_id, id);
            Some(comp)
        } else {
            self.comps.insert(comp_ind, (id, comp));
            None
        }
    }
    pub fn remove(&mut self, id: EntityHandle) -> Option<T> {
        assert!(!self.lending_removed_a_comp);

        if self.owners.contains(id.index) {
            self.owners.remove(id.index);
            let comp_ind = self.owners.count_ones(0..id.index); // exclude self
            let (id_out, comp) = self.comps.remove(comp_ind);
            assert_eq!(id_out, id);
            Some(comp)
        } else {
            None
        }
    }
    pub fn components(&self) -> &[(EntityHandle, T)] {
        &self.comps
    }
    pub fn iter(&self) -> impl Iterator<Item = &(EntityHandle, T)> {
        self.comps.iter()
    }
}

pub struct CompsIter;

#[derive(Default)]
pub struct World {
    handles: EntityHandleCounter,
    pub pos: ComponentStorage<Position>,
    pub vel: ComponentStorage<Velocity>,
}

impl World {
    pub fn new_entity(&mut self) -> EntityHandle {
        self.handles.next()
    }
    pub fn delete_entity(&mut self, entity: EntityHandle) {
        self.handles.entity_deleted();
        self.pos.remove(entity);
        self.vel.remove(entity);
    }
}

impl CompsIter {
    pub fn iter_comps<'a, T1, T2>(
        c1: &'a ComponentStorage<T1>,
        c2: &'a ComponentStorage<T2>,
    ) -> impl Iterator<Item = (EntityHandle, &'a T1, &'a T2)> {
        let mut inter = c1.owners.clone();
        inter.intersect_with(&c2.owners);
        inter.into_ones().map(|comp_ind| {
            let (id1, comp1) = &c1.components()[comp_ind];
            let (id2, comp2) = &c2.components()[comp_ind];
            assert_eq!(id1, id2);
            (*id1, comp1, comp2)
        })
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
