use crate::{ComponentStorage, CompsIter, EntityHandle};

impl<T> ComponentStorage<T> {
    fn tmp_remove_ind(&mut self, comp_ind: usize) -> (EntityHandle, T) {
        assert!(!self.lending_removed_a_comp);

        self.lending_removed_a_comp = true;
        let (id, comp) = self.comps.remove(comp_ind);
        self.owners.remove(id.index);
        (id, comp)
    }

    fn tmp_insert_ind(&mut self, id: EntityHandle, comp_ind: usize, comp: T) {
        assert!(self.lending_removed_a_comp);

        self.lending_removed_a_comp = false;
        self.owners.insert(id.index);
        self.comps.insert(comp_ind, (id, comp));
    }
}

impl CompsIter {
    /// This allows getting mutable references to components while still having mutable access to world
    /// For each entity with the right components, the function removes the components from their storage,
    /// passes them into the function parameter, then adds them back to their storage
    /// While the function is iterating, adding/removing components from the affected ComponentStorages
    /// is disabled, and will panic.
    pub fn lending_iter_comps<T1, T2, World>(
        world: &mut World,
        comp_func: impl Fn(&mut World) -> (&mut ComponentStorage<T1>, &mut ComponentStorage<T2>),
        func: impl Fn(EntityHandle, &mut T1, &mut T2, &mut World),
    ) {
        let (c1, c2) = comp_func(world);
        let mut inter = c1.owners.clone();
        inter.intersect_with(&c2.owners);

        inter.into_ones().for_each(|comp_ind| {
            let (c1, c2) = comp_func(world);
            let (id1, mut comp1) = c1.tmp_remove_ind(comp_ind);
            let (id2, mut comp2) = c2.tmp_remove_ind(comp_ind);
            assert_eq!(id1, id2);

            func(id1, &mut comp1, &mut comp2, world);

            let (c1, c2) = comp_func(world);
            c1.tmp_insert_ind(id1, comp_ind, comp1);
            c2.tmp_insert_ind(id1, comp_ind, comp2);
        })
    }
}
