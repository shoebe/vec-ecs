use crate::{
    comp_iter::CompIterHelper, CompIterHelperMut, EntityHandle, OptionalCombIterHelper,
    OptionalCombIterHelperMut,
};
use fixedbitset::FixedBitSet;

pub struct CompVec<T> {
    comps: Vec<(EntityHandle, T)>,
    owners: FixedBitSet,
}

impl<T> Default for CompVec<T> {
    fn default() -> Self {
        Self {
            comps: Default::default(),
            owners: Default::default(),
        }
    }
}

impl<T> CompVec<T> {
    pub fn get(&self, id: EntityHandle) -> Option<&T> {
        if self.owners.contains(id.index()) {
            let comp_ind = self.owners.count_ones(0..id.index()); // exclude self
            let (id_out, comp) = &self.comps[comp_ind];
            if *id_out == id {
                return Some(comp);
            }
        }
        None
    }

    pub fn get_mut(&mut self, id: EntityHandle) -> Option<&mut T> {
        if self.owners.contains(id.index()) {
            let comp_ind = self.owners.count_ones(0..id.index()); // exclude self
            let (id_out, comp) = &mut self.comps[comp_ind];
            if *id_out == id {
                return Some(comp);
            }
        }
        None
    }

    pub fn insert(&mut self, id: EntityHandle, comp: T) -> Option<T> {
        let already_had = self.owners.contains(id.index());

        self.owners.grow_and_insert(id.index());

        let comp_ind = self.owners.count_ones(0..id.index()); // exclude self

        if already_had {
            let (old_id, comp) = std::mem::replace(&mut self.comps[comp_ind], (id, comp));
            assert_eq!(old_id, id);
            Some(comp)
        } else {
            self.comps.insert(comp_ind, (id, comp));
            None
        }
    }
    pub fn remove(&mut self, id: EntityHandle) -> Option<T> {
        if self.owners.contains(id.index()) {
            self.owners.remove(id.index());
            let comp_ind = self.owners.count_ones(0..id.index()); // exclude self
            let (id_out, comp) = self.comps.remove(comp_ind);
            assert_eq!(id_out, id);
            Some(comp)
        } else {
            None
        }
    }
    pub fn components(&self) -> &[(EntityHandle, T)] {
        &self.comps
    }

    pub fn get_comp_ind(&self, id: usize) -> (EntityHandle, &T) {
        let (handle, comp) = &self.comps[id];
        (*handle, comp)
    }

    pub fn get_mut_comp_ind(&mut self, id: usize) -> (EntityHandle, &mut T) {
        let (handle, comp) = &mut self.comps[id];
        (*handle, comp)
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityHandle, &T)> {
        self.comps.iter().map(|(id, comp)| (*id, comp))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityHandle, &mut T)> {
        self.comps.iter_mut().map(|(id, comp)| (*id, comp))
    }

    pub fn owners(&self) -> &FixedBitSet {
        &self.owners
    }

    pub fn iter_helper(&self) -> CompIterHelper<T> {
        CompIterHelper::new(&self.comps, &self.owners)
    }
    pub fn iter_helper_mut(&mut self) -> CompIterHelperMut<T> {
        CompIterHelperMut::new(&mut self.comps, &self.owners)
    }

    pub fn optional_iter_helper(&self) -> OptionalCombIterHelper<T> {
        OptionalCombIterHelper(CompIterHelper::new(&self.comps, &self.owners))
    }
    pub fn optional_iter_helper_mut(&mut self) -> OptionalCombIterHelperMut<T> {
        OptionalCombIterHelperMut(CompIterHelperMut::new(&mut self.comps, &self.owners))
    }
}
