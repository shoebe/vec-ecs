use crate::EntityHandle;

pub trait EntityInsertIntoWorldTrait<WorldType> {
    fn insert_into_world(self, id: EntityHandle, world: &mut WorldType);
}

pub trait EntityBorrowFromWorldTrait<'a, WorldType>: Sized {
    fn borrow_from_world(entity_handle: EntityHandle, world: &'a mut WorldType) -> Self;
}
