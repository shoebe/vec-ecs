use fixedbitset::FixedBitSet;

use crate::{CompVec, EntityHandle, World};

macro_rules! iter_comps_combo {
    () => {};
}

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
    (@getfirstcompowners $first_comp:expr, $($tail:tt)* ) => {
        ($first_comp).owners().to_owned()
    };
    //iter_comps! { @ intersectwithothercompowners, __intersection, &mut world.pos, &mut world.vel; |id, pos, vel| {} }
    (@intersectwithothercompowners $var:expr, $first_comp:expr, $($comps:expr),*; $($tail:tt)*) => {
        $(
            $var.intersect_with(($comps).owners());
        )*
    };
    (@getfirstcomp $comp_ind:expr, &mut $e:expr, $($tail:tt)*) => {
        $e.get_mut_comp_ind($comp_ind)
    };
    (@getfirstcomp $comp_ind:expr, &$e:expr, $($tail:tt)*) => {
        $e.get_comp_ind($comp_ind)
    };
    (@getothercompstuple $id1:expr, $comp_ind:expr, $first:expr, $($tail:tt)* ) => {
        iter_comps!($id1, $comp_ind, $($tail)*)
    };
    (@getcompstuple $id1:expr, $comp_ind:expr, &mut $e:expr, $($tail:tt)* ) => {
        {
            let (id, comp) = $e.get_mut_comp_ind($comp_ind);
            assert_eq!(id, id1);
            comp
        }, iter_comps!(@getcomp $id1, $comp_ind, $($tail)*)
    };
    (@getcompstuple $id1:expr, $comp_ind:expr, &$e:expr, $($tail:tt)* ) => {
        {
            let (id, comp) = $e.get_comp_ind($comp_ind);
            assert_eq!(id, $id1);
            comp
        }, iter_comps!(@getcomp $id1, $comp_ind, $($tail)*)
    };
    (@get_func $($comps:expr),* ; $func:expr) => {
        $func
    };
    ($($tail:tt)*) => {{
        let mut __intersection = iter_comps!(@getfirstcompowners $($tail)*);

        iter_comps!(@intersectwithothercompowners __intersection, $($tail)*);

        __intersection.into_ones().for_each(|comp_ind| {
            let (id1, comp1) = iter_comps!(@getfirstcomp comp_ind, $($tail)*);
            let func = iter_comps!(@get_func $($tail)*);
            func(comp1, iter_comps!(@getcompstuple id1, comp_ind, $($tail)*));
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

        inter.into_ones().for_each(move |comp_ind| {
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
    trace_macros!(true);
    iter_comps!(&mut world.pos, &mut world.vel; |id, pos, vel| {

    });
}
