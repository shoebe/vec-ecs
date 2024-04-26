use crate::{entity_handle, CompVec, EntityHandle};

pub struct Iter<'a, T> {
    next_entity_ind: usize,
    owners: &'a fixedbitset::FixedBitSet,
    vec: &'a [(EntityHandle, T)],
}

impl<'a, T> Iter<'a, T> {
    pub(crate) fn new(vec: &'a [(EntityHandle, T)], owners: &'a fixedbitset::FixedBitSet) -> Self {
        Self {
            next_entity_ind: 0,
            vec,
            owners,
        }
    }
    pub fn advance_to(&mut self, entity_index: usize) {
        let advance_by = self.owners.count_ones(self.next_entity_ind..entity_index);
        self.vec = &self.vec[advance_by..];

        self.next_entity_ind = entity_index;
    }
    pub fn optional(self) -> Optional<Self> {
        Optional(self)
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (EntityHandle, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        match self.vec {
            [] => None,
            [(id, out), rest @ ..] => {
                self.vec = rest;
                self.next_entity_ind = id.index() + 1;
                Some((*id, out))
            }
        }
    }
}

pub struct IterMut<'a, T> {
    next_entity_ind: usize,
    owners: &'a fixedbitset::FixedBitSet,
    vec: &'a mut [(EntityHandle, T)],
}

impl<'a, T> IterMut<'a, T> {
    pub(crate) fn new(
        vec: &'a mut [(EntityHandle, T)],
        owners: &'a fixedbitset::FixedBitSet,
    ) -> Self {
        Self {
            next_entity_ind: 0,
            vec,
            owners,
        }
    }
    pub fn advance_to(&mut self, entity_index: usize) {
        let advance_by = self.owners.count_ones(self.next_entity_ind..entity_index);

        // from https://users.rust-lang.org/t/how-does-vecs-iterator-return-a-mutable-reference/60235/14
        let slice = std::mem::take(&mut self.vec);
        let (_prev, slice) = slice.split_at_mut(advance_by);
        self.vec = slice;

        self.next_entity_ind = entity_index;
    }

    pub fn optional(self) -> Optional<Self> {
        Optional(self)
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (EntityHandle, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        // from https://users.rust-lang.org/t/how-does-vecs-iterator-return-a-mutable-reference/60235/14
        let slice = std::mem::take(&mut self.vec);
        match slice {
            [] => None,
            [(id, out), rest @ ..] => {
                self.vec = rest;
                self.next_entity_ind = id.index() + 1;
                Some((*id, out))
            }
        }
    }
}

pub trait CompIterer {
    type Item;

    fn combine_owners(&self, owners: &mut fixedbitset::FixedBitSet);

    fn comp_at(&mut self, entity_handle: EntityHandle) -> Self::Item;
}

impl<'a, T> CompIterer for Iter<'a, T> {
    type Item = &'a T;

    fn combine_owners(&self, owners: &mut fixedbitset::FixedBitSet) {
        owners.intersect_with(self.owners);
    }

    fn comp_at(&mut self, entity_handle: EntityHandle) -> Self::Item {
        self.advance_to(entity_handle.index());
        let (handle2, comp) = self.next().unwrap();
        assert_eq!(entity_handle, handle2);
        comp
    }
}

impl<'a, T> CompIterer for IterMut<'a, T> {
    type Item = &'a mut T;

    fn combine_owners(&self, owners: &mut fixedbitset::FixedBitSet) {
        owners.intersect_with(self.owners);
    }

    fn comp_at(&mut self, entity_handle: EntityHandle) -> Self::Item {
        self.advance_to(entity_handle.index());
        let (handle2, comp) = self.next().unwrap();
        assert_eq!(entity_handle, handle2);
        comp
    }
}

pub trait NonOptionalCompIterer: CompIterer {
    fn owners(&self) -> &fixedbitset::FixedBitSet;
    fn comp_at_index(&mut self, entity_index: usize) -> (EntityHandle, Self::Item);
}

impl<'a, T> NonOptionalCompIterer for Iter<'a, T> {
    fn owners(&self) -> &fixedbitset::FixedBitSet {
        self.owners
    }

    fn comp_at_index(&mut self, entity_index: usize) -> (EntityHandle, Self::Item) {
        self.advance_to(entity_index);
        self.next().unwrap()
    }
}
impl<'a, T> NonOptionalCompIterer for IterMut<'a, T> {
    fn owners(&self) -> &fixedbitset::FixedBitSet {
        self.owners
    }
    fn comp_at_index(&mut self, entity_index: usize) -> (EntityHandle, Self::Item) {
        self.advance_to(entity_index);
        self.next().unwrap()
    }
}

pub struct Optional<T: NonOptionalCompIterer>(T);

impl<T: NonOptionalCompIterer> CompIterer for Optional<T> {
    type Item = Option<T::Item>;

    fn combine_owners(&self, _owners: &mut fixedbitset::FixedBitSet) {}

    fn comp_at(&mut self, entity_handle: EntityHandle) -> Self::Item {
        if self.0.owners().contains(entity_handle.index()) {
            Some(self.0.comp_at(entity_handle))
        } else {
            None
        }
    }
}

pub struct CompIter<T> {
    comps: T,
    owners: fixedbitset::FixedBitSet,
}

pub struct IntoCompIter<T> {
    comps: T,
    ones: fixedbitset::IntoOnes,
}

impl<T> CompIter<T> {
    #[must_use]
    pub fn without<Y>(mut self, without: &CompVec<Y>) -> Self {
        let mut inv = without.owners().to_owned();
        inv.grow(self.owners.len());
        inv.toggle_range(..);
        self.owners.intersect_with(&inv);
        self
    }
    #[must_use]
    pub fn with<Y>(mut self, with: &CompVec<Y>) -> Self {
        self.owners.intersect_with(with.owners());
        self
    }
}

//impl_iterer!(T2, T3, T4, T5 ; comp2, comp3, comp4, comp5)

macro_rules! impl_iterer {
    ($($generics:ty),* ; $($names:ident),*) => {};
}

impl<T1: NonOptionalCompIterer, T2: CompIterer, T3: CompIterer> CompIter<(T1, T2, T3)> {
    pub fn new(comp1: T1, comp2: T2, comp3: T3) -> Self {
        let mut owners = comp1.owners().to_owned();
        comp2.combine_owners(&mut owners);
        comp3.combine_owners(&mut owners);
        Self {
            comps: (comp1, comp2, comp3),
            owners,
        }
    }
}

impl<T1: NonOptionalCompIterer, T2: CompIterer, T3: CompIterer> IntoIterator
    for CompIter<(T1, T2, T3)>
{
    type IntoIter = IntoCompIter<(T1, T2, T3)>;
    type Item = <IntoCompIter<(T1, T2, T3)> as std::iter::Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        IntoCompIter {
            comps: self.comps,
            ones: self.owners.into_ones(),
        }
    }
}

impl<T1: NonOptionalCompIterer, T2: CompIterer, T3: CompIterer> Iterator
    for IntoCompIter<(T1, T2, T3)>
{
    type Item = (EntityHandle, T1::Item, T2::Item, T3::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.ones.next().map(|index| {
            let (id1, comp1) = self.comps.0.comp_at_index(index);
            (
                id1,
                comp1,
                self.comps.1.comp_at(id1),
                self.comps.2.comp_at(id1),
            )
        })
    }
}
