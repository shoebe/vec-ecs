use crate::{Entity, EntityBorrow, EntityHandle};

pub trait World: Sized {
    fn new_entity(&mut self) -> EntityHandle;
    fn delete_entity(&mut self, entity: EntityHandle);
    fn insert(&mut self, entity: impl Entity<WorldInsert = Self>) -> EntityHandle {
        let handle = self.new_entity();
        entity.insert_into_world(handle, self);
        handle
    }
}

pub trait WorldBorrow: Sized {
    fn borrow_entity<'a, T: EntityBorrow<WorldBorrow = &'a mut Self>>(
        &'a mut self,
        entity_handle: EntityHandle,
    ) -> T {
        T::borrow(entity_handle, self)
    }
}
