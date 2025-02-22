#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EntityHandle {
    index: usize,
    generation: u32,
}

impl EntityHandle {
    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

#[derive(Default, Debug)]
pub struct EntityHandleCounter {
    next: usize,
    generation: u32,
    removed: bool,
    free: fixedbitset::FixedBitSet,
    generation_to_free: Vec<Option<u32>>,
}

impl EntityHandleCounter {
    pub fn next_handle(&mut self) -> EntityHandle {
        if self.removed {
            self.removed = false;
            self.generation += 1;
        }
        let index = if let Some(ind) = self.free.minimum() {
            self.free.remove(ind);
            ind
        } else {
            let ind = self.next;
            self.next += 1;
            ind
        };
        if index >= self.generation_to_free.len() {
            self.generation_to_free.resize(index + 1, None);
        }
        self.generation_to_free[index] = Some(self.generation);
        EntityHandle {
            index,
            generation: self.generation,
        }
    }
    pub fn entity_deleted(&mut self, handle: EntityHandle) {
        assert!(!self.is_already_freed(handle));

        self.free.grow_and_insert(handle.index());
        self.removed = true;
        self.generation_to_free[handle.index()] = None;
    }

    pub fn is_already_freed(&self, handle: EntityHandle) -> bool {
        match self.generation_to_free[handle.index()] {
            None => true,
            Some(generation) => generation != handle.generation(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::EntityHandleCounter;

    #[test]
    fn test_counting() {
        let mut counter = EntityHandleCounter::default();
        let h1 = counter.next_handle();
        let h2 = counter.next_handle();
        assert_ne!(h1.index(), h2.index());
        assert_eq!(h1.generation(), h2.generation());

        assert!(!counter.is_already_freed(h1));
        counter.entity_deleted(h1);
        assert!(counter.is_already_freed(h1));
        let h3 = counter.next_handle();
        assert_eq!(h1.index(), h3.index());
        assert_ne!(h1.generation(), h3.generation());
        assert!(counter.is_already_freed(h1));
    }
}
