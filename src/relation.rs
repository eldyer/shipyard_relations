use petgraph::EdgeType;
use shipyard::{is_track_within_bounds, EntityId};

use crate::{
    iter::{BreadthFirstIter, DepthFirstIter},
    relation_mode::RelationMode,
    storage::RelationStorage,
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
    #[doc(hidden)]
    fn storage(&self) -> &RelationStorage<R>;
    #[doc(hidden)]
    fn last_insertion(&self) -> u32;
    #[doc(hidden)]
    fn last_deletion(&self) -> u32;
    #[doc(hidden)]
    fn current(&self) -> u32;

    fn get(&self, entity: EntityId) -> <<R as Relation>::Mode as RelationMode>::GetOutgoing<'_, R> {
        <<R as Relation>::Mode as RelationMode>::get_outgoing(&self.storage().graph, entity)
    }

    fn get_incoming(
        &self,
        entity: EntityId,
    ) -> <<R as Relation>::Mode as RelationMode>::GetIncoming<'_, R> {
        <<R as Relation>::Mode as RelationMode>::get_incoming(&self.storage().graph, entity)
    }

    fn get_outgoing(
        &self,
        entity: EntityId,
    ) -> <<R as Relation>::Mode as RelationMode>::GetOutgoing<'_, R> {
        <<R as Relation>::Mode as RelationMode>::get_outgoing(&self.storage().graph, entity)
    }

    fn relation(&self, a: EntityId, b: EntityId) -> Option<&R> {
        self.storage().graph.edge_weight(a, b)
    }

    fn is_inserted(&self, a: EntityId, b: EntityId) -> bool {
        self.storage().insertion_data.get(&(a, b)).map_or_else(
            || {
                if !<<R as Relation>::Mode as RelationMode>::EdgeType::is_directed() {
                    self.storage()
                        .insertion_data
                        .get(&(b, a))
                        .map_or(false, |timestamp| {
                            is_track_within_bounds(
                                *timestamp,
                                self.last_insertion(),
                                self.current(),
                            )
                        })
                } else {
                    false
                }
            },
            |timestamp| is_track_within_bounds(*timestamp, self.last_insertion(), self.current()),
        )
    }

    fn inserted(&self) -> Box<dyn Iterator<Item = (EntityId, EntityId)> + '_> {
        Box::new(
            self.storage()
                .insertion_data
                .iter()
                .filter(|(_, timestamp)| {
                    is_track_within_bounds(**timestamp, self.last_insertion(), self.current())
                })
                .map(|((a, b), _)| (*a, *b)),
        )
    }

    fn is_deleted(&self, a: EntityId, b: EntityId) -> bool {
        self.storage().deletion_data.get(&(a, b)).map_or_else(
            || {
                if !<<R as Relation>::Mode as RelationMode>::EdgeType::is_directed() {
                    self.storage()
                        .deletion_data
                        .get(&(b, a))
                        .map_or(false, |timestamp| {
                            is_track_within_bounds(
                                timestamp.0,
                                self.last_deletion(),
                                self.current(),
                            )
                        })
                } else {
                    false
                }
            },
            |timestamp| is_track_within_bounds(timestamp.0, self.last_deletion(), self.current()),
        )
    }

    fn deleted<'a>(&'a self) -> Box<dyn Iterator<Item = ((EntityId, EntityId), &'a R)> + 'a>
    where
        R: 'a,
    {
        Box::new(
            self.storage()
                .deletion_data
                .iter()
                .filter(|(_, timestamp)| {
                    is_track_within_bounds(timestamp.0, self.last_deletion(), self.current())
                })
                .map(|((a, b), (_, r))| ((*a, *b), r)),
        )
    }

    fn visit_depth_first(&self, entity: EntityId) -> DepthFirstIter<'_, R> {
        DepthFirstIter::new(&self.storage().graph, entity)
    }

    fn visit_breadth_first(&self, entity: EntityId) -> BreadthFirstIter<'_, R> {
        BreadthFirstIter::new(&self.storage().graph, entity)
    }
}
