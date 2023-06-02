use std::{
    any::{type_name, TypeId},
    error::Error,
    fmt::Formatter,
};

use petgraph::{
    algo::{is_cyclic_directed, is_cyclic_undirected},
    prelude::GraphMap,
    EdgeType,
};
use shipyard::*;

use crate::{relation_mode::RelationMode, storage::RelationStorage, GetRelation, Relation};

pub struct RelationViewMut<'a, R>
where
    R: Relation,
{
    pub(crate) storage: &'a mut RelationStorage<R>,
    _borrow: Option<ExclusiveBorrow<'a>>,
    _all_borrow: Option<SharedBorrow<'a>>,
    last_insertion: u32,
    last_deletion: u32,
    current: u32,
}

impl<R> Borrow for RelationViewMut<'_, R>
where
    R: Relation,
{
    type View<'a> = RelationViewMut<'a, R>;

    #[inline]
    fn borrow<'a>(
        all_storages: &'a AllStorages,
        all_borrow: Option<SharedBorrow<'a>>,
        last_run: Option<u32>,
        current: u32,
    ) -> Result<Self::View<'a>, error::GetStorage> {
        let view = all_storages.custom_storage_or_insert_mut(RelationStorage::<R>::default)?;

        let (storage, borrow) = unsafe { ARefMut::destructure(view) };

        let last_insertion = last_run.unwrap_or(storage.last_insert);
        let last_deletion = last_run.unwrap_or(current.wrapping_sub(u32::MAX / 2));

        Ok(RelationViewMut {
            storage,
            _borrow: Some(borrow),
            _all_borrow: all_borrow,
            last_insertion,
            last_deletion,
            current,
        })
    }
}

unsafe impl<R> BorrowInfo for RelationViewMut<'_, R>
where
    R: Relation,
{
    fn borrow_info(info: &mut Vec<info::TypeInfo>) {
        info.push(info::TypeInfo {
            name: type_name::<RelationStorage<R>>().into(),
            mutability: Mutability::Exclusive,
            storage_id: TypeId::of::<RelationStorage<R>>().into(),
            thread_safe: true,
        });
    }

    fn enable_tracking(
        _enable_tracking_fn: &mut Vec<fn(&AllStorages) -> Result<(), error::GetStorage>>,
    ) {
    }
}

impl<'a, R> Delete for RelationViewMut<'a, R>
where
    R: Relation,
{
    fn delete(&mut self, entity: EntityId) -> bool {
        self.storage.delete_node_tracked(entity, self.current)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InsertError {
    CycleDetected,
}

impl Error for InsertError {}

impl core::fmt::Debug for InsertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            InsertError::CycleDetected => {
                f.write_str("Tried to insert a relation that would cause a cyclic graph.")
            }
        }
    }
}

impl core::fmt::Display for InsertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        core::fmt::Debug::fmt(self, f)
    }
}

impl<'a, R> RelationViewMut<'a, R>
where
    R: Relation,
{
    pub fn insert(&mut self, a: EntityId, b: EntityId, relation: R) {
        self.insert_checked(a, b, relation).unwrap();
    }

    pub fn insert_checked(
        &mut self,
        a: EntityId,
        b: EntityId,
        relation: R,
    ) -> Result<(), InsertError> {
        if R::Mode::is_exclusive_incoming() {
            for e in self
                .storage
                .graph
                .neighbors_directed(b, petgraph::Direction::Incoming)
                .collect::<Vec<_>>()
            {
                assert!(self.storage.delete_edge_tracked(e, b, self.current));
            }
        } else if R::Mode::is_exclusive_outgoing() {
            for e in self
                .storage
                .graph
                .neighbors_directed(a, petgraph::Direction::Outgoing)
                .collect::<Vec<_>>()
            {
                self.storage.delete_edge_tracked(a, e, self.current);
            }
        }

        self.storage.insert_tracked(a, b, relation, self.current);

        let mut result = Ok(());

        if R::ACYCLIC {
            if <<R as Relation>::Mode as RelationMode>::EdgeType::is_directed() {
                if is_cyclic_directed(&self.storage.graph) {
                    result = Err(InsertError::CycleDetected);
                }
            } else if is_cyclic_undirected(&self.storage.graph) {
                result = Err(InsertError::CycleDetected);
            }
        }

        if result.is_err() {
            self.storage.graph.remove_edge(a, b); // do not track here
        }

        result
    }

    pub fn delete_relation(&mut self, a: EntityId, b: EntityId) -> bool {
        self.storage.delete_edge_tracked(a, b, self.current)
    }

    pub fn delete_relations_with(&mut self, e: EntityId) -> bool {
        self.storage.graph.remove_node(e)
    }

    pub fn is_inserted(&self, a: EntityId, b: EntityId) -> bool {
        self.storage
            .insertion_data
            .get(&(a, b))
            .map_or(false, |timestamp| {
                is_track_within_bounds(*timestamp, self.last_insertion, self.current)
            })
    }

    pub fn inserted(&self) -> impl Iterator<Item = (EntityId, EntityId)> + '_ {
        self.storage
            .insertion_data
            .iter()
            .filter(|(_, timestamp)| {
                is_track_within_bounds(**timestamp, self.last_insertion, self.current)
            })
            .map(|((a, b), _)| (*a, *b))
    }

    pub fn is_deleted(&self, a: EntityId, b: EntityId) -> bool {
        self.storage
            .deletion_data
            .get(&(a, b))
            .map_or(false, |timestamp| {
                is_track_within_bounds(timestamp.0, self.last_deletion, self.current)
            })
    }

    pub fn deleted(&self) -> impl Iterator<Item = ((EntityId, EntityId), &R)> + '_ {
        self.storage
            .deletion_data
            .iter()
            .filter(|(_, timestamp)| {
                is_track_within_bounds(timestamp.0, self.last_deletion, self.current)
            })
            .map(|((a, b), (_, r))| ((*a, *b), r))
    }

    pub fn clear_deleted(&mut self) {
        self.storage.deletion_data.clear();
    }
}

impl<R> GetRelation<R> for RelationViewMut<'_, R>
where
    R: Relation,
{
    fn graph(&self) -> &GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType> {
        &self.storage.graph
    }
}

impl<'a, R> GetRelation<R> for &'a RelationViewMut<'_, R>
where
    R: Relation,
{
    fn graph(&self) -> &GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType> {
        &self.storage.graph
    }
}

impl<'a, R> GetRelation<R> for &'a mut RelationViewMut<'_, R>
where
    R: Relation,
{
    fn graph(&self) -> &GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType> {
        &self.storage.graph
    }
}
