use crate::{CompVec, EntityHandle};

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
    /// Will not advance if entity_index is smaller than the last entity index
    fn advance_forward_to(&mut self, entity_index: usize) {
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

    /// Will not advance if entity_index is smaller than the last entity index
    fn advance_forward_to(&mut self, entity_index: usize) {
        let advance_by = self.owners.count_ones(self.next_entity_ind..entity_index);

        // from https://users.rust-lang.org/t/how-does-vecs-iterator-return-a-mutable-reference/60235/14
        let slice = std::mem::take(&mut self.vec);
        self.vec = &mut slice[advance_by..];

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
        self.advance_forward_to(entity_handle.index());
        let (handle2, comp) = self.next().unwrap();
        // TODO: asserting here is not ideal
        //       maybe have this function return a Result<Self::Item>?
        //       The iterer can iter again if it's err
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
        self.advance_forward_to(entity_handle.index());
        let (handle2, comp) = self.next().unwrap();
        // TODO: asserting here is not ideal
        //       maybe have this function return a Result<Self::Item>?
        //       The iterer can iter again if it's err
        assert_eq!(entity_handle, handle2);
        comp
    }
}

/// This trait is needed in `CompIter` since at least 1 of the
/// CompVecs needs to be non-optional in order to figure out
/// the ownership and the EntityHandle of the components
/// This was chosen to be the first CompVec in the tuple
pub trait NonOptionalCompIterer: CompIterer {
    fn owners(&self) -> &fixedbitset::FixedBitSet;
    fn comp_at_index(&mut self, entity_index: usize) -> (EntityHandle, Self::Item);
}

impl<'a, T> NonOptionalCompIterer for Iter<'a, T> {
    fn owners(&self) -> &fixedbitset::FixedBitSet {
        self.owners
    }

    fn comp_at_index(&mut self, entity_index: usize) -> (EntityHandle, Self::Item) {
        self.advance_forward_to(entity_index);
        self.next().unwrap()
    }
}
impl<'a, T> NonOptionalCompIterer for IterMut<'a, T> {
    fn owners(&self) -> &fixedbitset::FixedBitSet {
        self.owners
    }
    fn comp_at_index(&mut self, entity_index: usize) -> (EntityHandle, Self::Item) {
        self.advance_forward_to(entity_index);
        self.next().unwrap()
    }
}

/// Makes a CompVec<T> iterator return Option<T> instead.
/// Does not affect the ownership combination when used in
/// CompIter.
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

/// Iterator builder for components with shared ownership.
/// ```
/// # use vec_ecs::{CompVec, CompIter, EntityHandleCounter};
/// # let mut handles = EntityHandleCounter::default();
/// # let handle1 = handles.next_handle();
/// # let handle2 = handles.next_handle();
/// # let handle3 = handles.next_handle();
/// # let handle4 = handles.next_handle();
///
/// let mut v1 = CompVec::<u32>::default();
/// let mut v2 = CompVec::<bool>::default();
///
/// v1.insert(handle1, 100);
/// v2.insert(handle1, true);
///
/// v2.insert(handle2, false);
///
/// v1.insert(handle3, 32);
/// v2.insert(handle3, false);
///
/// let v: Vec<_> = CompIter::from((v1.iter(), v2.iter())).into_iter().collect();
/// assert_eq!(v, vec![(handle1, &100, &true), (handle3, &32, &false)]);
///  
/// ```
/// The first iterator cannot be an optional one.
pub struct CompIter<T> {
    comps: T,
    owners: fixedbitset::FixedBitSet,
}

impl<T> CompIter<T> {
    /// Exclude entities that have the specified components
    #[must_use]
    pub fn without<Y>(mut self, without: &CompVec<Y>) -> Self {
        self.owners.difference_with(without.owners());
        self
    }

    /// Only include entities that have the specified components
    #[must_use]
    pub fn with<Y>(mut self, with: &CompVec<Y>) -> Self {
        self.owners.intersect_with(with.owners());
        self
    }
}

/// The actualy iterator used by CompIter
pub struct IntoCompIter<T> {
    comps: T,
    ones: fixedbitset::IntoOnes,
}

macro_rules! impl_iterer {
    ($($generics:ident),* ; $($names:ident),*) => {
        impl<T1: NonOptionalCompIterer, $($generics: CompIterer, )*> From<(T1, $($generics),*)> for CompIter<(T1, $($generics),*)> {
            fn from(comps: (T1, $($generics),*)) -> Self {
                let (
                    comp1,
                    $(
                        $names,
                    )*
                ) = comps;

                #[allow(unused_mut)]
                let mut owners = comp1.owners().to_owned();
                $(
                    $names.combine_owners(&mut owners);
                )*
                Self {
                    comps: (comp1, $($names,)*),
                    owners,
                }
            }
        }

        impl<T1: NonOptionalCompIterer, $($generics: CompIterer, )* > IntoIterator
            for CompIter<(T1, $($generics),*)>
        {
            type IntoIter = IntoCompIter<(T1, $($generics),*)>;
            type Item = <IntoCompIter<(T1, $($generics),*)> as std::iter::Iterator>::Item;

            fn into_iter(self) -> Self::IntoIter {
                IntoCompIter {
                    comps: self.comps,
                    ones: self.owners.into_ones(),
                }
            }
        }

        impl<T1: NonOptionalCompIterer, $($generics: CompIterer, )* > Iterator
            for IntoCompIter<(T1, $($generics),*)>
        {
            type Item = (EntityHandle, T1::Item, $($generics::Item, )*);

            fn next(&mut self) -> Option<Self::Item> {
                self.ones.next().map(|index| {
                    let (
                        comp1,
                        $(
                            $names,
                        )*
                    ) = &mut self.comps;
                    let (id1, comp1) = comp1.comp_at_index(index);
                    (
                        id1,
                        comp1,
                        $(
                            $names.comp_at(id1),
                        )*
                    )
                })
            }
        }
    };
}
impl_iterer!(;);
impl_iterer!(T2; comp2);
impl_iterer!(T2, T3; comp2, comp3);
impl_iterer!(T2, T3, T4; comp2, comp3, comp4);
impl_iterer!(T2, T3, T4, T5; comp2, comp3, comp4, comp5);
impl_iterer!(T2, T3, T4, T5, T6; comp2, comp3, comp4, comp5, comp6);
impl_iterer!(T2, T3, T4, T5, T6, T7; comp2, comp3, comp4, comp5, comp6, comp7);
