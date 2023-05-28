use std::collections::HashSet;

use petgraph::{
    prelude::GraphMap,
    visit::{Bfs, Dfs},
};
use shipyard::*;

use crate::{Relation, RelationMode};

pub trait RelationsIter<R>
where
    R: Relation,
{
    fn visit_depth_first(&self, entity: EntityId) -> DepthFirstIter<R>;
    fn visit_breadth_first(&self, entity: EntityId) -> BreadthFirstIter<R>;
}

pub struct DepthFirstIter<'a, R>
where
    R: Relation,
{
    graph: &'a GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType>,
    dfs: Dfs<EntityId, HashSet<EntityId>>,
}

impl<R> Iterator for DepthFirstIter<'_, R>
where
    R: Relation,
{
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        self.dfs.next(self.graph)
    }
}

impl<'a, R> DepthFirstIter<'a, R>
where
    R: Relation,
{
    pub fn new(
        graph: &'a GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType>,
        start: EntityId,
    ) -> Self {
        DepthFirstIter {
            graph,
            dfs: Dfs::new(graph, start),
        }
    }
}

pub struct BreadthFirstIter<'a, R>
where
    R: Relation,
{
    graph: &'a GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType>,
    bfs: Bfs<EntityId, HashSet<EntityId>>,
}

impl<R> Iterator for BreadthFirstIter<'_, R>
where
    R: Relation,
{
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        self.bfs.next(self.graph)
    }
}

impl<'a, R> BreadthFirstIter<'a, R>
where
    R: Relation,
{
    pub fn new(
        graph: &'a GraphMap<EntityId, R, <<R as Relation>::Mode as RelationMode>::EdgeType>,
        start: EntityId,
    ) -> Self {
        BreadthFirstIter {
            graph,
            bfs: Bfs::new(graph, start),
        }
    }
}
