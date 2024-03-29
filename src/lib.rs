//! Shipyard Relations provides Entity Relations for [Shipyard](https://github.com/leudz/shipyard).

/// Iterator types and trait.
pub mod iter;
mod relation;
mod relation_ext;
/// Modes to control directedness and exclusiveness of a relation type.
pub mod relation_mode;
mod storage;
mod view;
mod view_mut;

#[doc(inline)]
pub use self::iter::RelationsIter;
pub use self::{
    relation::{GetRelation, Relation},
    relation_ext::RelationExt,
    storage::RelationStorage,
    view::RelationView,
    view_mut::{InsertError, RelationViewMut},
};
#[doc(hidden)]
pub use petgraph::prelude::GraphMap;

#[test]
fn test_undirected_cyclic() {
    use shipyard::*;

    use crate::{relation_mode::Undirected, Relation, RelationViewMut};

    #[derive(Debug)]
    struct Foo;

    impl Relation for Foo {
        type Mode = Undirected;

        const ACYCLIC: bool = false;
    }

    let mut world = World::new();

    let e0 = world.add_entity(());
    let e1 = world.add_entity(());
    let e2 = world.add_entity(());
    let e3 = world.add_entity(());
    let e4 = world.add_entity(());
    let e5 = world.add_entity(());

    let mut r_foo = world.borrow::<RelationViewMut<Foo>>().unwrap();

    r_foo.insert(e0, e0, Foo);
    r_foo.insert(e0, e1, Foo);
    r_foo.insert(e0, e2, Foo);
    r_foo.insert(e2, e3, Foo);
    r_foo.insert(e3, e4, Foo);

    assert_eq!(
        r_foo.get(e0).map(|e| e.0).collect::<Vec<_>>(),
        vec![e0, e1, e2]
    );

    assert_eq!(
        r_foo.visit_depth_first(e0).collect::<Vec<_>>(),
        vec![e0, e2, e3, e4, e1]
    );

    assert_eq!(
        r_foo.visit_breadth_first(e0).collect::<Vec<_>>(),
        vec![e0, e1, e2, e3, e4]
    );

    drop(r_foo);

    #[derive(Default, Unique)]
    struct Inserted(Vec<(EntityId, EntityId)>);

    #[derive(Default, Unique)]
    struct Deleted(Vec<(EntityId, EntityId)>);

    world.add_unique(Inserted::default());
    world.add_unique(Deleted::default());

    let system = move |r_foo: RelationViewMut<Foo>,
                       mut inserted: UniqueViewMut<Inserted>,
                       mut deleted: UniqueViewMut<Deleted>| {
        inserted.0.clear();
        inserted.0.extend(r_foo.inserted());
        deleted.0.clear();
        deleted.0.extend(r_foo.deleted().map(|((a, b), _)| (a, b)));
    };

    let workload = move || -> Workload { system.into_workload() };

    world.add_workload(workload);

    world.run_workload(workload).unwrap();

    assert_eq!(
        world.borrow::<UniqueView<Inserted>>().unwrap().0,
        vec![(e0, e0), (e0, e1), (e0, e2), (e2, e3), (e3, e4)]
    );

    world
        .borrow::<RelationViewMut<Foo>>()
        .unwrap()
        .insert(e4, e5, Foo);

    world.delete_entity(e2);

    world.run_workload(workload).unwrap();

    assert_eq!(
        world.borrow::<UniqueView<Inserted>>().unwrap().0,
        vec![(e4, e5)]
    );

    assert_eq!(
        world.borrow::<UniqueView<Deleted>>().unwrap().0,
        vec![(e0, e2), (e3, e2)]
    );

    world.run_workload(workload).unwrap();

    assert_eq!(world.borrow::<UniqueView<Inserted>>().unwrap().0, vec![]);

    assert_eq!(world.borrow::<UniqueView<Deleted>>().unwrap().0, vec![]);
}

#[test]
fn test_directed() {
    use shipyard::*;

    use crate::{relation_mode::Directed, Relation, RelationViewMut};

    #[derive(Debug)]
    struct Foo;

    impl Relation for Foo {
        type Mode = Directed;
    }

    let mut world = World::new();

    let e0 = world.add_entity(());
    let e1 = world.add_entity(());
    let e2 = world.add_entity(());
    let e3 = world.add_entity(());
    let e4 = world.add_entity(());

    let mut r_foo = world.borrow::<RelationViewMut<Foo>>().unwrap();

    r_foo.insert(e0, e1, Foo);
    r_foo.insert(e0, e2, Foo);
    r_foo.insert(e2, e3, Foo);
    r_foo.insert(e3, e4, Foo);

    assert_eq!(r_foo.get(e0).map(|e| e.0).collect::<Vec<_>>(), vec![e1, e2]);

    assert_eq!(
        r_foo.visit_depth_first(e0).collect::<Vec<_>>(),
        vec![e0, e2, e3, e4, e1]
    );

    assert_eq!(
        r_foo.visit_breadth_first(e0).collect::<Vec<_>>(),
        vec![e0, e1, e2, e3, e4]
    );

    assert_eq!(
        r_foo.get_outgoing_inserted(e0).collect::<Vec<_>>(),
        vec![e1, e2]
    );

    assert_eq!(r_foo.get_incoming_inserted(e0).collect::<Vec<_>>(), vec![]);

    assert_eq!(
        r_foo.get_incoming_inserted(e3).collect::<Vec<_>>(),
        vec![e2]
    );

    r_foo.delete(e3);

    assert_eq!(
        r_foo
            .get_outgoing_deleted(e3)
            .map(|e| e.0)
            .collect::<Vec<_>>(),
        vec![e4]
    );

    assert_eq!(
        r_foo
            .get_incoming_deleted(e3)
            .map(|e| e.0)
            .collect::<Vec<_>>(),
        vec![e2]
    );
}

#[test]
fn test_directed_exclusive() {
    use shipyard::*;

    use crate::{relation_mode::DirectedExclusive, Relation, RelationViewMut};

    #[derive(Debug)]
    struct Foo;

    impl Relation for Foo {
        type Mode = DirectedExclusive;
    }

    let mut world = World::new();

    let e0 = world.add_entity(());
    let e1 = world.add_entity(());
    let e2 = world.add_entity(());
    let e3 = world.add_entity(());

    let mut r_foo = world.borrow::<RelationViewMut<Foo>>().unwrap();

    r_foo.insert(e0, e1, Foo);
    r_foo.insert(e0, e2, Foo);
    r_foo.insert(e0, e3, Foo);

    assert_eq!(r_foo.get_outgoing(e0).map(|e| e.0), Some(e3));

    assert_eq!(
        r_foo.get_outgoing_inserted(e0).collect::<Vec<_>>(),
        vec![e3]
    );

    assert_eq!(
        r_foo
            .get_outgoing_deleted(e0)
            .map(|e| e.0)
            .collect::<Vec<_>>(),
        vec![e1, e2]
    );
}
