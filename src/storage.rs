use indexmap::IndexMap;
use petgraph::prelude::GraphMap;
use shipyard::*;

use crate::{relation_mode::RelationMode, Relation};

pub struct RelationStorage<R>
where
    R: Relation,
{
    pub(crate) graph: GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType>,
    pub(crate) last_insert: u32,
    pub(crate) insertion_data: IndexMap<(EntityId, EntityId), u32>,
    pub(crate) deletion_data: IndexMap<(EntityId, EntityId), (u32, R)>,
}

impl<R> Default for RelationStorage<R>
where
    R: Relation,
{
    fn default() -> Self {
        Self {
            graph: GraphMap::default(),
            last_insert: 0,
            insertion_data: IndexMap::new(),
            deletion_data: IndexMap::new(),
        }
    }
}

impl<R> Storage for RelationStorage<R>
where
    R: Relation,
{
    fn memory_usage(&self) -> Option<StorageMemoryUsage> {
        // TODO
        None
    }

    fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    fn delete(&mut self, entity: EntityId, current: u32) {
        self.delete_node_tracked(entity, current);
    }
}

impl<R> RelationStorage<R>
where
    R: Relation,
{
    pub(crate) fn delete_edge_tracked(&mut self, a: EntityId, b: EntityId, current: u32) -> bool {
        if let Some(r) = self.graph.remove_edge(a, b) {
            self.insertion_data.remove(&(a, b));
            self.deletion_data.insert((a, b), (current, r));
            true
        } else {
            false
        }
    }

    pub(crate) fn delete_node_tracked(&mut self, entity: EntityId, current: u32) -> bool {
        for e in self
            .graph
            .neighbors_directed(entity, petgraph::Direction::Incoming)
            .collect::<Vec<_>>()
        {
            self.delete_edge_tracked(e, entity, current);
        }

        for e in self
            .graph
            .neighbors_directed(entity, petgraph::Direction::Outgoing)
            .collect::<Vec<_>>()
        {
            self.delete_edge_tracked(entity, e, current);
        }

        self.graph.remove_node(entity)
    }

    pub(crate) fn insert_tracked(&mut self, a: EntityId, b: EntityId, relation: R, current: u32) {
        if self.graph.add_edge(a, b, relation).is_none() {
            self.insertion_data.insert((a, b), current);
        }
    }
}
