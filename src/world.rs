use crate::{EntityBorrowFromWorldTrait, EntityHandle, EntityInsertIntoWorldTrait};

pub trait WorldTrait: WorldBorrowTrait<'static> {
    fn delete_entity(&mut self, entity: EntityHandle);
    fn insert_at(&mut self, handle: EntityHandle, entity: impl EntityInsertIntoWorldTrait<Self>) {
        entity.insert_into_world(handle, self)
    }
    fn insert(&mut self, entity: impl EntityInsertIntoWorldTrait<Self>) -> EntityHandle {
        let handle = self.new_entity();
        entity.insert_into_world(handle, self);
        handle
    }
    fn is_empty(&self) -> bool;
}

pub trait WorldBorrowTrait<'a>: Sized {
    fn new_entity(&mut self) -> EntityHandle;
    fn is_entity_already_freed(&self, handle: EntityHandle) -> bool;
    fn borrow_entity<T: EntityBorrowFromWorldTrait<'a, Self>>(
        &'a mut self,
        entity_handle: EntityHandle,
    ) -> T {
        T::borrow_from_world(entity_handle, self)
    }
}
