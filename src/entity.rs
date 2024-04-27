use crate::EntityHandle;

pub trait Entity {
    type WorldInsert;
    fn insert_into_world(self, id: EntityHandle, world: &mut Self::WorldInsert);
}

pub trait EntityBorrow: Sized {
    type WorldBorrow;
    fn borrow(handle: EntityHandle, world: Self::WorldBorrow) -> Self;
}
