use crate::EntityHandle;
use fixedbitset::FixedBitSet;

#[derive(Debug)]
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

    pub fn get2_mut(
        &mut self,
        id1: EntityHandle,
        id2: EntityHandle,
    ) -> (Option<&mut T>, Option<&mut T>) {
        match (
            self.owners.contains(id1.index()),
            self.owners.contains(id2.index()),
        ) {
            (true, true) => {
                let comp_ind1 = self.owners.count_ones(0..id1.index());
                let comp_ind2 = self.owners.count_ones(0..id2.index());
                let max = comp_ind1.max(comp_ind2);
                let (slice_min, slice_max) = self.comps.split_at_mut(max);
                let (comp1, comp2) = if comp_ind1 < comp_ind2 {
                    (&mut slice_min[comp_ind1], &mut slice_max[0])
                } else {
                    (&mut slice_max[0], &mut slice_min[comp_ind2])
                };
                (
                    (comp1.0 == id1).then_some(&mut comp1.1),
                    (comp2.0 == id2).then_some(&mut comp2.1),
                )
            }
            (true, false) => (self.get_mut(id1), None),
            (false, true) => (None, self.get_mut(id2)),
            (false, false) => (None, None),
        }
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

    pub fn iter(&self) -> crate::Iter<'_, T> {
        crate::Iter::new(&self.comps, &self.owners)
    }

    pub fn iter_mut(&mut self) -> crate::IterMut<'_, T> {
        crate::IterMut::new(&mut self.comps, &self.owners)
    }

    pub fn owners(&self) -> &FixedBitSet {
        &self.owners
    }

    pub fn is_empty(&self) -> bool {
        self.comps.is_empty()
    }
}

#[cfg(test)]
mod test {
    use crate::{CompVec, EntityHandleCounter};

    #[test]
    fn test_get2_mut() {
        let mut v = CompVec::<String>::default();
        let mut handles = EntityHandleCounter::default();
        let id1 = handles.next_handle();
        v.insert(id1, "hello".to_string());
        let id2 = handles.next_handle();
        v.insert(id2, "hello2".to_string());
        let id3 = handles.next_handle();
        v.insert(id3, "hello3".to_string());

        {
            let (s1, s2) = v.get2_mut(id1, id2);
            assert_eq!(s1, Some(&mut "hello".to_string()));
            assert_eq!(s2, Some(&mut "hello2".to_string()));
        }

        {
            let (s1, s3) = v.get2_mut(id1, id3);
            assert_eq!(s1, Some(&mut "hello".to_string()));
            assert_eq!(s3, Some(&mut "hello3".to_string()));
        }

        {
            let (s3, s1) = v.get2_mut(id3, id1);
            assert_eq!(s1, Some(&mut "hello".to_string()));
            assert_eq!(s3, Some(&mut "hello3".to_string()));
        }

        {
            let (s3, s2) = v.get2_mut(id3, id2);
            assert_eq!(s2, Some(&mut "hello2".to_string()));
            assert_eq!(s3, Some(&mut "hello3".to_string()));
        }
    }
}
