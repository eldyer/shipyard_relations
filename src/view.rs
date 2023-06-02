use std::any::{type_name, TypeId};

use petgraph::prelude::GraphMap;
use shipyard::*;

use crate::{relation_mode::RelationMode, storage::RelationStorage, GetRelation, Relation};

pub struct RelationView<'a, R>
where
    R: Relation,
{
    pub(crate) storage: &'a RelationStorage<R>,
    _borrow: Option<SharedBorrow<'a>>,
    _all_borrow: Option<SharedBorrow<'a>>,
    last_insertion: u32,
    last_deletion: u32,
    current: u32,
}

impl<R> Borrow for RelationView<'_, R>
where
    R: Relation,
{
    type View<'a> = RelationView<'a, R>;

    #[inline]
    fn borrow<'a>(
        all_storages: &'a AllStorages,
        all_borrow: Option<SharedBorrow<'a>>,
        last_run: Option<u32>,
        current: u32,
    ) -> Result<Self::View<'a>, error::GetStorage> {
        let view = all_storages.custom_storage_or_insert(RelationStorage::<R>::default)?;

        let (storage, borrow) = unsafe { ARef::destructure(view) };

        let last_insertion = last_run.unwrap_or(storage.last_insert);
        let last_deletion = last_run.unwrap_or(current.wrapping_sub(u32::MAX / 2));

        Ok(RelationView {
            storage,
            _borrow: Some(borrow),
            _all_borrow: all_borrow,
            last_insertion,
            last_deletion,
            current,
        })
    }
}

unsafe impl<R> BorrowInfo for RelationView<'_, R>
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

impl<R> RelationView<'_, R>
where
    R: Relation,
{
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
}

impl<R> GetRelation<R> for RelationView<'_, R>
where
    R: Relation,
{
    fn graph(&self) -> &GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType> {
        &self.storage.graph
    }
}

impl<R> GetRelation<R> for &RelationView<'_, R>
where
    R: Relation,
{
    fn graph(&self) -> &GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType> {
        &self.storage.graph
    }
}
