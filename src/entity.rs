use crate::EntityHandle;

pub trait EntityTrait {
    type WorldInsert;
    fn insert_into_world(self, id: EntityHandle, world: &mut Self::WorldInsert);
}

pub trait EntityBorrowTrait<'a, WorldBorrow>: Sized {
    fn from_world(entity_handle: EntityHandle, world: &'a mut WorldBorrow) -> Self;
}
