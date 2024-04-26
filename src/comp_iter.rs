use fixedbitset::FixedBitSet;

use crate::{CompVec, EntityHandle, World};

pub struct CompIter;

impl CompIter {
    pub fn iter_comps<'a, T1, T2>(
        c1: &'a CompVec<T1>,
        c2: &'a CompVec<T2>,
    ) -> impl Iterator<Item = (EntityHandle, &'a T1, &'a T2)> {
        let mut inter = c1.owners().clone();
        inter.intersect_with(c2.owners());
        inter.into_ones().map(|comp_ind| {
            let (id1, comp1) = c1.get_comp_ind(comp_ind);
            let (id2, comp2) = c2.get_comp_ind(comp_ind);
            // could do filter_map instead of assert here
            assert_eq!(id1, id2);
            (id1, comp1, comp2)
        })
    }
}

macro_rules! iter_comps {
    (@first_expr $e:expr, $($tail:tt)*) => {
        $e
    };
    (@getfirstcomp $comp_ind:expr, &mut $e:expr, $($tail:tt)*) => {
        (&mut $e).get_mut_comp_ind($comp_ind)
    };
    (@getfirstcomp $comp_ind:expr, &$e:expr, $($tail:tt)*) => {
        (& $e).get_comp_ind($comp_ind)
    };
    (@intersections $var:expr, $comp1_skip:expr, $($comps:expr),*; $func:expr) => {
        $(
            $var.intersect_with(($comps).owners());
        )*
    };
    (@func $($comps:expr),*; $func:expr) => {
        $func
    };
    (@tailcomp_skip_first $comp_ind:expr, $id1:expr, $comp1:expr, $($tail:tt)*) => {
        iter_comps!(@tailcomp $comp_ind, $id1, $($tail)*)
    };
    (@tailcomp $comp_ind:expr, $id1:expr, & $comp:expr; $func:expr) => {
        {
            let (id, comp) = (& $comp).get_comp_ind($comp_ind);
            assert_eq!($id1, id);
            comp
        }
    };
    (@tailcomp $comp_ind:expr, $id1:expr, &mut $comp:expr; $func:expr) => {
        {
            let (id, comp) = (&mut $comp).get_mut_comp_ind($comp_ind);
            assert_eq!($id1, id);
            comp
        }
    };
    (@tailcomp $comp_ind:expr, $id1:expr, & $comp:expr, $($tail:tt)*) => {
        (
            {
                let (id, comp) = (& $comp).get_comp_ind($comp_ind);
                assert_eq!($id1, id);
                comp
            }, iter_comps!(@tailcomp $comp_ind, $id1, $($tail)*)
        )
    };
    (@tailcomp $comp_ind:expr, $id1:expr, &mut $comp:expr, $($tail:tt)*) => {
        (
            {
                let (id, comp) = (&mut $comp).get_mut_comp_ind($comp_ind);
                assert_eq!($id1, id);
                comp
            }, iter_comps!(@tailcomp $comp_ind, $id1, $($tail)*)
        )
    };
    ($($tts:tt)*) => {{
        let mut __intersection = iter_comps!(@first_expr $($tts)*).owners().to_owned();

        iter_comps!(@intersections __intersection, $($tts)*);

        __intersection.into_ones().for_each(|comp_ind| {
            let (id1, comp1) = iter_comps!(@getfirstcomp comp_ind, $($tts)*);

            let func = iter_comps!(@func $($tts)*);
            func(
                id1,
                (
                    comp1,
                    iter_comps!(@tailcomp_skip_first comp_ind, id1, $($tts)*),
                )
            );
        })
    }};
}

pub fn test(world: &mut World) {
    //for (a, b, c) in iter_comps!(&mut world.pos, &world.vel, &mut world.yomama, Without(&world.kek)) {
    //
    //}
    {
        let mut inter = world.pos.owners().clone();
        inter.intersect_with(world.vel.owners());
        inter.intersect_with(world.yomama.owners());

        {
            let mut c = world.excluded.owners().to_owned();
            c.toggle_range(..);
            inter.intersect_with(&c);
        }

        inter.into_ones().for_each(|comp_ind| {
            let (id1, comp1) = world.pos.get_mut_comp_ind(comp_ind);
            let (id2, comp2) = world.vel.get_comp_ind(comp_ind);
            let (id3, comp3) = world.yomama.get_mut_comp_ind(comp_ind);
            // could do filter_map instead of assert here
            assert_eq!(id1, id2);
            assert_eq!(id1, id3);

            (|(id1, comp1, comp2, comp3)| {})((id1, comp1, comp2, comp3));
        })
    }
    {}

    iter_comps!(&mut world.pos, &mut world.vel, &mut world.yomama; |id, (pos, (vel, yomama))| {

    });
}
