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

#[derive(Default)]
pub struct EntityHandleCounter {
    next: usize,
    gen: u32,
    removed: bool,
}

impl EntityHandleCounter {
    pub fn next_handle(&mut self) -> EntityHandle {
        if self.removed {
            self.removed = false;
            self.gen += 1;
        }
        let n = EntityHandle {
            index: self.next,
            gen: self.gen,
        };
        self.next += 1;
        n
    }
    pub fn entity_deleted(&mut self) {
        self.removed = true
    }
}
