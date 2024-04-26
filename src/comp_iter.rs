use crate::EntityHandle;

pub struct CompIterHelper<'a, T> {
    last: usize,
    owners: &'a fixedbitset::FixedBitSet,
    vec: &'a [(EntityHandle, T)],
}

impl<'a, T> CompIterHelper<'a, T> {
    pub fn new(vec: &'a [(EntityHandle, T)], owners: &'a fixedbitset::FixedBitSet) -> Self {
        Self {
            last: 0,
            vec,
            owners,
        }
    }
    pub fn comp_at(&mut self, entity_index: usize) -> (EntityHandle, &'a T) {
        let comp_ind = self.owners.count_ones(self.last..entity_index);
        self.vec = &self.vec[comp_ind..];
        match self.vec {
            [] => panic!(),
            [(id, out), rest @ ..] => {
                self.vec = rest;
                self.last = entity_index + 1;
                (*id, out)
            }
        }
    }
}

pub struct CompIterHelperMut<'a, T> {
    last: usize,
    owners: &'a fixedbitset::FixedBitSet,
    vec: &'a mut [(EntityHandle, T)],
}

impl<'a, T> CompIterHelperMut<'a, T> {
    pub fn new(vec: &'a mut [(EntityHandle, T)], owners: &'a fixedbitset::FixedBitSet) -> Self {
        Self {
            last: 0,
            vec,
            owners,
        }
    }
    pub fn comp_at(&mut self, entity_index: usize) -> (EntityHandle, &'a mut T) {
        let comp_ind = self.owners.count_ones(self.last..entity_index);

        // from https://users.rust-lang.org/t/how-does-vecs-iterator-return-a-mutable-reference/60235/14
        // not entirely sure why this works but I'll take it
        let slice = std::mem::take(&mut self.vec);
        let (_prev, slice) = slice.split_at_mut(comp_ind);
        match slice {
            [] => panic!(),
            [(id, out), rest @ ..] => {
                self.vec = rest;
                self.last = entity_index + 1;
                (*id, out)
            }
        }
    }
}

pub struct OptionalCombIterHelper<'a, T>(pub CompIterHelper<'a, T>);

impl<'a, T> OptionalCombIterHelper<'a, T> {
    pub fn comp_at(&mut self, entity_index: usize) -> Option<(EntityHandle, &'a T)> {
        if self.0.owners.contains(entity_index) {
            Some(self.0.comp_at(entity_index))
        } else {
            None
        }
    }
}

pub struct OptionalCombIterHelperMut<'a, T>(pub CompIterHelperMut<'a, T>);

impl<'a, T> OptionalCombIterHelperMut<'a, T> {
    pub fn comp_at(&mut self, entity_index: usize) -> Option<(EntityHandle, &'a mut T)> {
        if self.0.owners.contains(entity_index) {
            Some(self.0.comp_at(entity_index))
        } else {
            None
        }
    }
}
