use shipyard::{AllStorages, EntityId, World};

use crate::{InsertError, Relation, RelationViewMut};

/// Extension trait for adding relations using `shipyard::World` or `shipyard::AllStorages`.
pub trait RelationExt {
    fn add_relation<R>(&self, a: EntityId, b: EntityId, relation: R) -> Result<(), InsertError>
    where
        R: Relation;

    fn add_relation_unchecked<R>(&self, a: EntityId, b: EntityId, relation: R)
    where
        R: Relation;
}

impl RelationExt for World {
    fn add_relation<R>(&self, a: EntityId, b: EntityId, relation: R) -> Result<(), InsertError>
    where
        R: Relation,
    {
        let mut relation_view = self.borrow::<RelationViewMut<R>>().unwrap();
        relation_view.insert_checked(a, b, relation)
    }

    fn add_relation_unchecked<R>(&self, a: EntityId, b: EntityId, relation: R)
    where
        R: Relation,
    {
        let mut relation_view = self.borrow::<RelationViewMut<R>>().unwrap();
        relation_view.insert(a, b, relation);
    }
}

impl RelationExt for AllStorages {
    fn add_relation<R>(&self, a: EntityId, b: EntityId, relation: R) -> Result<(), InsertError>
    where
        R: Relation,
    {
        let mut relation_view = self.borrow::<RelationViewMut<R>>().unwrap();
        relation_view.insert_checked(a, b, relation)
    }

    fn add_relation_unchecked<R>(&self, a: EntityId, b: EntityId, relation: R)
    where
        R: Relation,
    {
        let mut relation_view = self.borrow::<RelationViewMut<R>>().unwrap();
        relation_view.insert(a, b, relation);
    }
}
