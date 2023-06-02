use petgraph::{prelude::GraphMap, EdgeType};
use shipyard::EntityId;

#[doc(hidden)]
pub trait RelationMode {
    type EdgeType: EdgeType + Send + Sync + 'static;
    type GetIncoming<'a, R>
    where
        R: 'a;
    type GetOutgoing<'a, R>
    where
        R: 'a;

    fn is_exclusive_incoming() -> bool;
    fn is_exclusive_outgoing() -> bool;

    fn get<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetOutgoing<'_, R> {
        Self::get_outgoing(graph, entity)
    }

    fn get_incoming<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetIncoming<'_, R>;

    fn get_outgoing<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetOutgoing<'_, R>;
}

pub enum Directed {}
pub enum DirectedExclusive {}
pub enum DirectedExclusiveIncoming {}
pub enum DirectedExclusiveOutgoing {}
pub enum Undirected {}
pub enum UndirectedExclusive {}

impl RelationMode for Directed {
    type EdgeType = petgraph::Directed;
    type GetIncoming<'a, R> = Box<dyn Iterator<Item = (EntityId, &'a R)> + 'a> where R: 'a;
    type GetOutgoing<'a, R> = Box<dyn Iterator<Item = (EntityId, &'a R)> + 'a> where R: 'a;

    fn is_exclusive_incoming() -> bool {
        false
    }

    fn is_exclusive_outgoing() -> bool {
        false
    }

    fn get_incoming<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetIncoming<'_, R> {
        Box::new(
            graph
                .edges_directed(entity, petgraph::Direction::Incoming)
                .map(|(e, _, r)| (e, r)),
        )
    }

    fn get_outgoing<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetOutgoing<'_, R> {
        Box::new(
            graph
                .edges_directed(entity, petgraph::Direction::Outgoing)
                .map(|(_, e, r)| (e, r)),
        )
    }
}

impl RelationMode for DirectedExclusive {
    type EdgeType = petgraph::Directed;
    type GetIncoming<'a, R> = Option<(EntityId, &'a R)> where R: 'a;
    type GetOutgoing<'a, R> = Option<(EntityId, &'a R)> where R: 'a;

    fn is_exclusive_incoming() -> bool {
        true
    }

    fn is_exclusive_outgoing() -> bool {
        true
    }

    fn get_incoming<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetIncoming<'_, R> {
        graph
            .edges_directed(entity, petgraph::Direction::Incoming)
            .map(|(e, _, r)| (e, r))
            .next()
    }

    fn get_outgoing<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetOutgoing<'_, R> {
        graph
            .edges_directed(entity, petgraph::Direction::Outgoing)
            .map(|(_, e, r)| (e, r))
            .next()
    }
}

impl RelationMode for DirectedExclusiveIncoming {
    type EdgeType = petgraph::Directed;
    type GetIncoming<'a, R> = Option<(EntityId, &'a R)> where R: 'a;
    type GetOutgoing<'a, R> = Box<dyn Iterator<Item = (EntityId, &'a R)> + 'a> where R: 'a;

    fn is_exclusive_incoming() -> bool {
        true
    }

    fn is_exclusive_outgoing() -> bool {
        false
    }

    fn get_incoming<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetIncoming<'_, R> {
        graph
            .edges_directed(entity, petgraph::Incoming)
            .map(|(e, _, r)| (e, r))
            .next()
    }

    fn get_outgoing<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetOutgoing<'_, R> {
        Box::new(
            graph
                .edges_directed(entity, petgraph::Direction::Outgoing)
                .map(|(_, e, r)| (e, r)),
        )
    }
}

impl RelationMode for DirectedExclusiveOutgoing {
    type EdgeType = petgraph::Directed;
    type GetIncoming<'a, R> = Box<dyn Iterator<Item = (EntityId, &'a R)> + 'a> where R: 'a;
    type GetOutgoing<'a, R> = Option<(EntityId, &'a R)> where R: 'a;

    fn is_exclusive_incoming() -> bool {
        false
    }

    fn is_exclusive_outgoing() -> bool {
        true
    }

    fn get_incoming<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetIncoming<'_, R> {
        Box::new(
            graph
                .edges_directed(entity, petgraph::Direction::Incoming)
                .map(|(e, _, r)| (e, r)),
        )
    }

    fn get_outgoing<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetOutgoing<'_, R> {
        graph
            .edges_directed(entity, petgraph::Direction::Outgoing)
            .map(|(_, e, r)| (e, r))
            .next()
    }
}

impl RelationMode for Undirected {
    type EdgeType = petgraph::Undirected;
    type GetIncoming<'a, R> = Box<dyn Iterator<Item = (EntityId, &'a R)> + 'a> where R: 'a;
    type GetOutgoing<'a, R> = Box<dyn Iterator<Item = (EntityId, &'a R)> + 'a> where R: 'a;

    fn is_exclusive_incoming() -> bool {
        false
    }

    fn is_exclusive_outgoing() -> bool {
        false
    }

    fn get<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetOutgoing<'_, R> {
        Box::new(graph.edges(entity).map(move |(e1, e2, r)| match entity {
            e if e == e1 => (e2, r),
            e if e == e2 => (e1, r),
            _ => unreachable!(),
        }))
    }

    fn get_incoming<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetIncoming<'_, R> {
        Box::new(
            graph
                .edges_directed(entity, petgraph::Direction::Incoming)
                .map(|(e, _, r)| (e, r)),
        )
    }

    fn get_outgoing<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetOutgoing<'_, R> {
        Box::new(
            graph
                .edges_directed(entity, petgraph::Direction::Outgoing)
                .map(|(_, e, r)| (e, r)),
        )
    }
}

impl RelationMode for UndirectedExclusive {
    type EdgeType = petgraph::Undirected;
    type GetIncoming<'a, R> = Option<(EntityId, &'a R)> where R: 'a;
    type GetOutgoing<'a, R> = Option<(EntityId, &'a R)> where R: 'a;

    fn is_exclusive_incoming() -> bool {
        true
    }

    fn is_exclusive_outgoing() -> bool {
        true
    }

    fn get<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetOutgoing<'_, R> {
        graph
            .edges(entity)
            .map(move |(e1, e2, r)| match entity {
                e if e == e1 => (e2, r),
                e if e == e2 => (e1, r),
                _ => unreachable!(),
            })
            .next()
    }

    fn get_incoming<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetIncoming<'_, R> {
        graph
            .edges_directed(entity, petgraph::Direction::Incoming)
            .map(|(e, _, r)| (e, r))
            .next()
    }

    fn get_outgoing<R>(
        graph: &GraphMap<EntityId, R, <Self as RelationMode>::EdgeType>,
        entity: EntityId,
    ) -> <Self as RelationMode>::GetIncoming<'_, R> {
        graph
            .edges_directed(entity, petgraph::Direction::Outgoing)
            .map(|(_, e, r)| (e, r))
            .next()
    }
}
