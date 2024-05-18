use vec_ecs::{CompIter, CompVec, EntityHandleCounter, WorldTrait};

#[derive(Debug, Default)]
pub struct One(usize);

#[derive(Debug, Default)]
pub struct Two(usize);

#[derive(Debug, Default)]
pub struct Three(usize);

#[derive(vec_ecs::World, Default)]
pub struct World {
    #[world(handles)]
    handles: EntityHandleCounter,
    pub one: CompVec<One>,
    pub two: CompVec<Two>,
    pub three: CompVec<Three>,
}

#[test]
#[allow(dead_code)]
fn test_add_remove() {
    #[derive(vec_ecs::Entity, Debug)]
    #[entity(insert = World)]
    pub struct TestEntity {
        pub one: One,
        pub two: Two,
        pub three: Three,
    }

    let mut world = World::default();
    assert!(world.is_empty());
    let mut es: Vec<_> = (0..10)
        .map(|i| {
            let e = world.insert(TestEntity {
                one: One(i),
                two: Two(i),
                three: Three(i),
            });
            let val = i;
            (e, val)
        })
        .collect();

    for (ind, (e, val)) in es.iter().enumerate() {
        assert_eq!(ind, e.index());
        assert_eq!(ind, *val);
    }

    for ((id, one, two, three), (expected_e, expected_val)) in
        CompIter::from((world.one.iter(), world.two.iter(), world.three.iter()))
            .into_iter()
            .zip(es.iter())
    {
        assert_eq!(id, *expected_e);
        assert_eq!(one.0, *expected_val);
        assert_eq!(two.0, *expected_val);
        assert_eq!(three.0, *expected_val);
    }

    let remove = [0, 3, 2];

    for remove in remove {
        let (e, _val) = es.remove(remove);
        world.delete_entity(e);
    }

    for ((id, one, two, three), (expected_e, expected_val)) in
        CompIter::from((world.one.iter(), world.two.iter(), world.three.iter()))
            .into_iter()
            .zip(es.iter())
    {
        assert_eq!(id, *expected_e);
        assert_eq!(one.0, *expected_val);
        assert_eq!(two.0, *expected_val);
        assert_eq!(three.0, *expected_val);
    }

    // 0, 2->3, 3->4
    let add = [0, 3, 4];

    for (ind, add) in add.iter().enumerate() {
        let val = 9 + ind;
        let e = world.insert(TestEntity {
            one: One(val),
            two: Two(val),
            three: Three(val),
        });
        assert_eq!(e.index(), *add);
        es.push((e, val));
    }
    es.sort_by_key(|(e, _val)| e.index());

    dbg!(&es);

    let v: Vec<_> = CompIter::from((world.one.iter(), world.two.iter(), world.three.iter()))
        .into_iter()
        .collect();

    dbg!(v);

    for ((id, one, two, three), (expected_e, expected_val)) in
        CompIter::from((world.one.iter(), world.two.iter(), world.three.iter()))
            .into_iter()
            .zip(es.iter())
    {
        assert_eq!(id, *expected_e);
        assert_eq!(one.0, *expected_val);
        assert_eq!(two.0, *expected_val);
        assert_eq!(three.0, *expected_val);
    }
}
