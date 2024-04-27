#[test]
fn test_borrow() {
    fn borrow_1(thing: &mut String) -> &mut String {
        thing
    }
    let mut s = "hello".to_string();
    let s2 = borrow_1(&mut s);
    dbg!(s2);
    dbg!(s);
}

#[test]
fn test_borrow2() {
    #[derive(Debug)]
    struct Borrow<'a> {
        s: &'a mut String,
    }
    fn borrow_2(thing: &mut String) -> Borrow<'_> {
        Borrow { s: thing }
    }
    let mut s = "hello".to_string();
    let s2 = borrow_2(&mut s);
    dbg!(s2);
    dbg!(s);
}

#[test]
fn test_borrow3() {
    #[derive(Debug)]
    struct Borrow<'a> {
        s: &'a mut String,
    }
    #[derive(Debug)]
    struct Borrow2<'a> {
        s: &'a mut String,
    }
    fn borrow_3<'b, 'a: 'b>(borrow: &'b mut Borrow<'a>) -> Borrow2<'b> {
        Borrow2 { s: borrow.s }
    }
    let mut s = "hello".to_string();
    let mut b = Borrow { s: &mut s };
    let b2 = borrow_3(&mut b);
    dbg!(&b2);
    dbg!(&b);
    dbg!(&s);
}

#[test]
fn test_borrow4() {
    #[derive(Debug)]
    struct EntityBorrowStruct<'a> {
        s: &'a mut String,
    }

    #[derive(Debug)]
    struct WorldBorrowStruct<'a> {
        s: &'a mut String,
    }

    impl<'a, 'b: 'a> EntityBorrowStruct<'a> {
        fn borrow_from(world: &'a mut WorldBorrowStruct<'b>) -> Self {
            Self { s: world.s }
        }
    }

    let mut s = "hello".to_string();
    let mut world = WorldBorrowStruct { s: &mut s };
    let entity = EntityBorrowStruct::borrow_from(&mut world);
    dbg!(&entity);
    dbg!(&world);
    dbg!(&s);
}

#[test]
fn test_borrow5() {
    #[derive(Debug)]
    struct EntityBorrowStruct<'a> {
        s: &'a mut String,
    }

    #[derive(Debug)]
    struct WorldBorrowStruct<'a> {
        s: &'a mut String,
    }

    impl<'a, 'b: 'a> EntityBorrowStruct<'a> {
        fn borrow_from(world: &'a mut WorldBorrowStruct<'b>) -> Self {
            Self { s: world.s }
        }
    }

    impl<'a, 'b: 'a> WorldBorrowStruct<'b> {
        fn borrow_entity(&'a mut self) -> EntityBorrowStruct<'a> {
            EntityBorrowStruct::borrow_from(self)
        }
    }

    let mut s = "hello".to_string();
    let mut world = WorldBorrowStruct { s: &mut s };
    let entity: EntityBorrowStruct = world.borrow_entity();
    dbg!(&entity);
    dbg!(&world);
    dbg!(&s);
}

#[test]
fn test_borrow6() {
    #[derive(Debug)]
    struct EntityBorrowStruct<'a> {
        s: &'a mut String,
    }

    #[derive(Debug)]
    struct WorldBorrowStruct<'a> {
        s: &'a mut String,
    }

    trait EntityBorrow<'a, WorldBorrow>: Sized {
        fn borrow(world: &'a mut WorldBorrow) -> Self;
    }

    impl<'a, 'b: 'a> EntityBorrow<'a, WorldBorrowStruct<'b>> for EntityBorrowStruct<'a> {
        fn borrow(world: &'a mut WorldBorrowStruct<'b>) -> Self {
            Self { s: world.s }
        }
    }

    impl<'a, 'b: 'a> WorldBorrowStruct<'b> {
        fn borrow_entity(&'a mut self) -> EntityBorrowStruct<'a> {
            EntityBorrowStruct::borrow(self)
        }
    }

    let mut s = "hello".to_string();
    let mut world = WorldBorrowStruct { s: &mut s };
    let entity: EntityBorrowStruct = world.borrow_entity();
    dbg!(&entity);
    dbg!(&world);
    dbg!(&s);
}

#[test]
fn test_borrow7() {
    #[derive(Debug)]
    struct EntityBorrowStruct<'a> {
        s: &'a mut String,
    }

    #[derive(Debug)]
    struct WorldBorrowStruct<'a> {
        s: &'a mut String,
    }

    trait EntityBorrow<'a, WorldBorrow>: Sized {
        fn borrow(world: &'a mut WorldBorrow) -> Self;
    }

    impl<'a, 'b: 'a> EntityBorrow<'a, WorldBorrowStruct<'b>> for EntityBorrowStruct<'a> {
        fn borrow(world: &'a mut WorldBorrowStruct<'b>) -> Self {
            Self { s: world.s }
        }
    }

    trait WorldBorrow<'a>: Sized {
        fn borrow_entity<T: EntityBorrow<'a, Self>>(&'a mut self) -> T {
            T::borrow(self)
        }
    }

    impl<'a, 'b: 'a> WorldBorrow<'a> for WorldBorrowStruct<'b> {}

    let mut s = "hello".to_string();
    let mut world = WorldBorrowStruct { s: &mut s };
    let entity: EntityBorrowStruct = world.borrow_entity();
    dbg!(&entity);
    dbg!(&world);
    dbg!(&s);
}
