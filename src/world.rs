use crate::{EntityBorrowFromWorldTrait, EntityHandle, EntityInsertIntoWorldTrait};

pub trait WorldTrait: Sized {
    fn new_entity(&mut self) -> EntityHandle;
    fn delete_entity(&mut self, entity: EntityHandle);
    fn insert(&mut self, entity: impl EntityInsertIntoWorldTrait<Self>) -> EntityHandle {
        let handle = self.new_entity();
        entity.insert_into_world(handle, self);
        handle
    }
    fn is_empty(&self) -> bool;
}

pub trait WorldBorrowTrait<'a>: Sized {
    fn new_entity_from_borrow(&mut self) -> EntityHandle;
    fn borrow_entity<T: EntityBorrowFromWorldTrait<'a, Self>>(
        &'a mut self,
        entity_handle: EntityHandle,
    ) -> T {
        T::borrow_from_world(entity_handle, self)
    }
}
