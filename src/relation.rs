use petgraph::EdgeType;
use shipyard::{EntityId, TrackingTimestamp};

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
    fn last_insertion(&self) -> TrackingTimestamp;
    #[doc(hidden)]
    fn last_deletion(&self) -> TrackingTimestamp;
    #[doc(hidden)]
    fn current(&self) -> TrackingTimestamp;

    fn get(&self, entity: EntityId) -> <R::Mode as RelationMode>::GetOutgoing<'_, R> {
        <R::Mode as RelationMode>::get_outgoing(&self.storage().graph, entity)
    }

    fn get_incoming(&self, entity: EntityId) -> <R::Mode as RelationMode>::GetIncoming<'_, R> {
        <R::Mode as RelationMode>::get_incoming(&self.storage().graph, entity)
    }

    fn get_outgoing(&self, entity: EntityId) -> <R::Mode as RelationMode>::GetOutgoing<'_, R> {
        <R::Mode as RelationMode>::get_outgoing(&self.storage().graph, entity)
    }

    fn get_inserted(&self, entity: EntityId) -> Box<dyn Iterator<Item = EntityId> + '_> {
        self.get_outgoing_inserted(entity)
    }

    fn get_outgoing_inserted(&self, entity: EntityId) -> Box<dyn Iterator<Item = EntityId> + '_> {
        let iter = self.inserted();
        if <R::Mode as RelationMode>::EdgeType::is_directed() {
            Box::new(iter.filter_map(move |(a, b)| (a == entity).then_some(b)))
        } else {
            Box::new(iter.filter_map(move |(a, b)| {
                if a == entity {
                    Some(b)
                } else if b == entity {
                    Some(a)
                } else {
                    None
                }
            }))
        }
    }

    fn get_incoming_inserted(&self, entity: EntityId) -> Box<dyn Iterator<Item = EntityId> + '_> {
        let iter = self.inserted();
        if <R::Mode as RelationMode>::EdgeType::is_directed() {
            Box::new(iter.filter_map(move |(a, b)| (b == entity).then_some(a)))
        } else {
            Box::new(iter.filter_map(move |(a, b)| {
                if b == entity {
                    Some(a)
                } else if a == entity {
                    Some(b)
                } else {
                    None
                }
            }))
        }
    }

    fn get_deleted<'a>(
        &'a self,
        entity: EntityId,
    ) -> Box<dyn Iterator<Item = (EntityId, &'a R)> + 'a> {
        self.get_outgoing_deleted(entity)
    }

    fn get_outgoing_deleted<'a>(
        &'a self,
        entity: EntityId,
    ) -> Box<dyn Iterator<Item = (EntityId, &'a R)> + 'a> {
        let iter = self.deleted();
        if <R::Mode as RelationMode>::EdgeType::is_directed() {
            Box::new(iter.filter_map(move |((a, b), r)| (a == entity).then_some((b, r))))
        } else {
            Box::new(iter.filter_map(move |((a, b), r)| {
                if a == entity {
                    Some((b, r))
                } else if b == entity {
                    Some((a, r))
                } else {
                    None
                }
            }))
        }
    }

    fn get_incoming_deleted<'a>(
        &'a self,
        entity: EntityId,
    ) -> Box<dyn Iterator<Item = (EntityId, &'a R)> + 'a> {
        let iter = self.deleted();
        if <R::Mode as RelationMode>::EdgeType::is_directed() {
            Box::new(iter.filter_map(move |((a, b), r)| (b == entity).then_some((a, r))))
        } else {
            Box::new(iter.filter_map(move |((a, b), r)| {
                if b == entity {
                    Some((a, r))
                } else if a == entity {
                    Some((b, r))
                } else {
                    None
                }
            }))
        }
    }

    fn relation(&self, a: EntityId, b: EntityId) -> Option<&R> {
        self.storage().graph.edge_weight(a, b)
    }

    fn is_inserted(&self, a: EntityId, b: EntityId) -> bool {
        self.storage().insertion_data.get(&(a, b)).map_or_else(
            || {
                if !<R::Mode as RelationMode>::EdgeType::is_directed() {
                    self.storage()
                        .insertion_data
                        .get(&(b, a))
                        .map_or(false, |timestamp| {
                            timestamp.is_within(self.last_insertion(), self.current())
                        })
                } else {
                    false
                }
            },
            |timestamp| timestamp.is_within(self.last_insertion(), self.current()),
        )
    }

    fn inserted(&self) -> Box<dyn Iterator<Item = (EntityId, EntityId)> + '_> {
        Box::new(
            self.storage()
                .insertion_data
                .iter()
                .filter(|(_, timestamp)| timestamp.is_within(self.last_insertion(), self.current()))
                .map(|((a, b), _)| (*a, *b)),
        )
    }

    fn is_deleted(&self, a: EntityId, b: EntityId) -> bool {
        self.storage().deletion_data.get(&(a, b)).map_or_else(
            || {
                if !<R::Mode as RelationMode>::EdgeType::is_directed() {
                    self.storage()
                        .deletion_data
                        .get(&(b, a))
                        .map_or(false, |timestamp| {
                            timestamp.0.is_within(self.last_deletion(), self.current())
                        })
                } else {
                    false
                }
            },
            |timestamp| timestamp.0.is_within(self.last_deletion(), self.current()),
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
                    timestamp.0.is_within(self.last_deletion(), self.current())
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
