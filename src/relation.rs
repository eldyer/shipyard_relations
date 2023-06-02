use petgraph::prelude::GraphMap;
use shipyard::EntityId;

use crate::{
    iter::{BreadthFirstIter, DepthFirstIter},
    relation_mode::RelationMode,
};

/// Indicates that a `struct` or `enum` is used as a relation type.
pub trait Relation: Send + Sync + 'static + Sized {
    type Mode: RelationMode + Send + Sync + 'static;

    const ACYCLIC: bool = true;
}

/// Used to retrieve various information from a relation view.
pub trait GetRelation<R>
where
    R: Relation,
{
    fn graph(&self) -> &GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType>;

    fn get(&self, entity: EntityId) -> <<R as Relation>::Mode as RelationMode>::GetOutgoing<'_, R> {
        <<R as Relation>::Mode as RelationMode>::get_outgoing(self.graph(), entity)
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

    fn relation(&self, a: EntityId, b: EntityId) -> Option<&R> {
        self.graph().edge_weight(a, b)
    }

    fn visit_depth_first(&self, entity: EntityId) -> DepthFirstIter<'_, R> {
        DepthFirstIter::new(self.graph(), entity)
    }

    fn visit_breadth_first(&self, entity: EntityId) -> BreadthFirstIter<'_, R> {
        BreadthFirstIter::new(self.graph(), entity)
    }
}
