use crate::EntityHandle;

pub struct CompIterHelper<'a, T> {
    last: usize,
    owners: &'a fixedbitset::FixedBitSet,
    vec: T,
}

impl<'a, T> CompIterHelper<'a, &'a [(EntityHandle, T)]> {
    pub fn new(vec: &'a [(EntityHandle, T)], owners: &'a fixedbitset::FixedBitSet) -> Self {
        Self {
            last: 0,
            vec,
            owners,
        }
    }
    pub fn comp_at(&mut self, ind: usize) -> (EntityHandle, &'a T) {
        let comp_ind = self.owners.count_ones(self.last..ind);
        self.vec = &self.vec[comp_ind..];
        match self.vec {
            [] => panic!(),
            [(id, out), rest @ ..] => {
                self.vec = rest;
                self.last = ind + 1;
                (*id, out)
            }
        }
    }
}

impl<'a, T> CompIterHelper<'a, &'a mut [(EntityHandle, T)]> {
    pub fn new_mut(vec: &'a mut [(EntityHandle, T)], owners: &'a fixedbitset::FixedBitSet) -> Self {
        Self {
            last: 0,
            vec,
            owners,
        }
    }
    pub fn comp_at(&mut self, ind: usize) -> (EntityHandle, &'a mut T) {
        let comp_ind = self.owners.count_ones(self.last..ind);

        // from https://users.rust-lang.org/t/how-does-vecs-iterator-return-a-mutable-reference/60235/14
        // not entirely sure why this works but I'll take it
        let slice = std::mem::take(&mut self.vec);
        let (prev, slice) = slice.split_at_mut(comp_ind);
        match slice {
            [] => panic!(),
            [(id, out), rest @ ..] => {
                self.vec = rest;
                self.last = ind + 1;
                (*id, out)
            }
        }
    }
}

pub struct DualIterator<'a, T1, T2> {
    ones: fixedbitset::IntoOnes,
    it1: CompIterHelper<'a, &'a mut [(EntityHandle, T1)]>,
    it2: CompIterHelper<'a, &'a [(EntityHandle, T2)]>,
}
impl<'a, T1, T2> Iterator for DualIterator<'a, T1, T2> {
    type Item = (EntityHandle, &'a mut T1, &'a T2);

    fn next(&mut self) -> Option<Self::Item> {
        self.ones.next().map(move |index| {
            let (id1, comp1) = self.it1.comp_at(index);
            let (id2, comp2) = self.it2.comp_at(index);
            (id1, comp1, comp2)
        })
    }
}
