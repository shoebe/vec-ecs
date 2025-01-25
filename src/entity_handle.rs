#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EntityHandle {
    index: usize,
    gen: u32,
}

impl EntityHandle {
    pub fn gen(&self) -> u32 {
        self.gen
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

#[derive(Default, Debug)]
pub struct EntityHandleCounter {
    next: usize,
    gen: u32,
    removed: bool,
    free: fixedbitset::FixedBitSet,
}

impl EntityHandleCounter {
    pub fn next_handle(&mut self) -> EntityHandle {
        if self.removed {
            self.removed = false;
            self.gen += 1;
        }
        let index = if let Some(ind) = self.free.minimum() {
            self.free.remove(ind);
            ind
        } else {
            let ind = self.next;
            self.next += 1;
            ind
        };
        EntityHandle {
            index,
            gen: self.gen,
        }
    }
    pub fn entity_deleted(&mut self, handle: EntityHandle) {
        self.free.grow_and_insert(handle.index());
        self.removed = true
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
        assert_eq!(h1.gen(), h2.gen());

        counter.entity_deleted(h1);
        let h3 = counter.next_handle();
        assert_eq!(h1.index(), h3.index());
        assert_ne!(h1.gen(), h3.gen());
    }
}
