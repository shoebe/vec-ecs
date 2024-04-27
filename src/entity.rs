use crate::EntityHandle;

pub trait Entity {
    type WorldInsert;
    fn insert_into_world(self, id: EntityHandle, world: &mut Self::WorldInsert);
}

pub trait EntityBorrow<'a, WorldBorrow>: Sized {
    fn borrow(entity_handle: EntityHandle, world: &'a mut WorldBorrow) -> Self;
}
