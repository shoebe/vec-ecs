use crate::{EntityBorrowTrait, EntityHandle, EntityTrait};

pub trait WorldTrait: Sized {
    fn new_entity(&mut self) -> EntityHandle;
    fn delete_entity(&mut self, entity: EntityHandle);
    fn insert(&mut self, entity: impl EntityTrait<WorldInsert = Self>) -> EntityHandle {
        let handle = self.new_entity();
        entity.insert_into_world(handle, self);
        handle
    }
}

pub trait WorldBorrowTrait<'a>: Sized {
    fn borrow_entity<T: EntityBorrowTrait<'a, Self>>(
        &'a mut self,
        entity_handle: EntityHandle,
    ) -> T {
        T::borrow(entity_handle, self)
    }
}
