use crate::EntityHandle;

pub trait EntityTrait {
    type WorldInsert;
    fn insert_into_world(self, id: EntityHandle, world: &mut Self::WorldInsert);
}

pub trait EntityBorrowTrait<'a, WorldType>: Sized {
    fn borrow_from_world(entity_handle: EntityHandle, world: &'a mut WorldType) -> Self;
}
