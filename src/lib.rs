mod iter;
mod relation;
mod storage;
mod view;
mod view_mut;

pub use self::iter::{BreadthFirstIter, DepthFirstIter, RelationsIter};
pub use self::relation::{
    Directed, DirectedExclusive, DirectedExclusiveIncoming, DirectedExclusiveOutgoing, Relation,
    RelationMode, Undirected, UndirectedExclusive,
};
pub use self::storage::RelationStorage;
pub use self::view::RelationView;
pub use self::view_mut::{InsertError, RelationViewMut};
pub use petgraph::prelude::GraphMap;

use shipyard::EntityId;

pub trait GetGraph<R>
where
    R: Relation,
{
    fn graph(&self) -> &GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType>;

    fn get(&self, entity: EntityId) -> <<R as Relation>::Mode as RelationMode>::GetOutgoing<'_, R> {
        <<R as Relation>::Mode as RelationMode>::get_outgoing(self.graph(), entity)
    }

    fn relation(&self, a: EntityId, b: EntityId) -> Option<&R> {
        self.graph().edge_weight(a, b)
    }

    fn get_incoming(
        &self,
        entity: EntityId,
    ) -> <<R as Relation>::Mode as RelationMode>::GetIncoming<'_, R> {
        <<R as Relation>::Mode as RelationMode>::get_incoming(self.graph(), entity)
    }

    fn get_outgoing(
        &self,
        entity: EntityId,
    ) -> <<R as Relation>::Mode as RelationMode>::GetOutgoing<'_, R> {
        <<R as Relation>::Mode as RelationMode>::get_outgoing(self.graph(), entity)
    }

    fn visit_depth_first(&self, entity: EntityId) -> DepthFirstIter<'_, R> {
        DepthFirstIter::new(self.graph(), entity)
    }

    fn visit_breadth_first(&self, entity: EntityId) -> BreadthFirstIter<'_, R> {
        BreadthFirstIter::new(self.graph(), entity)
    }
}

#[test]
fn test_undirected_cyclic() {
    use shipyard::*;

    use crate::{Relation, RelationViewMut};

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
