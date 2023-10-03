use std::any::{type_name, TypeId};

use shipyard::*;

use crate::{storage::RelationStorage, GetRelation, Relation};

/// Shared view over a relation storage.
pub struct RelationView<'a, R>
where
    R: Relation,
{
    pub(crate) storage: &'a RelationStorage<R>,
    _borrow: Option<SharedBorrow<'a>>,
    _all_borrow: Option<SharedBorrow<'a>>,
    last_insertion: TrackingTimestamp,
    last_deletion: TrackingTimestamp,
    current: TrackingTimestamp,
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
        last_run: Option<TrackingTimestamp>,
        current: TrackingTimestamp,
    ) -> Result<Self::View<'a>, error::GetStorage> {
        let view = all_storages.custom_storage_or_insert(|| RelationStorage::<R>::new(current))?;

        let (storage, borrow) = unsafe { ARef::destructure(view) };

        let last_insertion = last_run.unwrap_or(storage.last_insert);
        let last_deletion = last_run.unwrap_or(current.furthest_from());

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

impl<R> GetRelation<R> for RelationView<'_, R>
where
    R: Relation,
{
    fn storage(&self) -> &RelationStorage<R> {
        self.storage
    }
    fn last_insertion(&self) -> TrackingTimestamp {
        self.last_insertion
    }
    fn last_deletion(&self) -> TrackingTimestamp {
        self.last_deletion
    }
    fn current(&self) -> TrackingTimestamp {
        self.current
    }
}

impl<R> GetRelation<R> for &RelationView<'_, R>
where
    R: Relation,
{
    fn storage(&self) -> &RelationStorage<R> {
        self.storage
    }
    fn last_insertion(&self) -> TrackingTimestamp {
        self.last_insertion
    }
    fn last_deletion(&self) -> TrackingTimestamp {
        self.last_deletion
    }
    fn current(&self) -> TrackingTimestamp {
        self.current
    }
}
