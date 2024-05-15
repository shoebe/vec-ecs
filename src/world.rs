use crate::{EntityHandle, EntityTrait};

pub trait WorldTrait: Sized {
    fn new_entity(&mut self) -> EntityHandle;
    fn delete_entity(&mut self, entity: EntityHandle);
    fn insert(&mut self, entity: impl EntityTrait<WorldInsert = Self>) -> EntityHandle {
        let handle = self.new_entity();
        entity.insert_into_world(handle, self);
        handle
    }
    fn is_empty(&self) -> bool;
}
