//! to help with generating a graph of 2D grid mesh to run pathfinding on
//!

use petgraph::graphmap::GraphMap;

use crate::transforms::{SSBox, SSPoint, SSVec};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GridNode(pub SSPoint);

/// two vertices are equal if their coordinates are equal
impl PartialOrd for GridNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// ord of points based on their x - y coordinate tuple
impl Ord for GridNode {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0.x, self.0.y).cmp(&(other.0.x, other.0.y))
    }
}

impl GridNode {
    /// returns the neighbors of this node
    fn neighbors(&self) -> Vec<Self> {
        vec![
            Self(self.0 + SSVec::new(-1, 0)),
            Self(self.0 + SSVec::new(0, -1)),
            Self(self.0 + SSVec::new(1, 0)),
            Self(self.0 + SSVec::new(0, 1)),
        ]
    }
}

const DIM: SSBox = SSBox::new(SSPoint::new(-500, -500), SSPoint::new(500, 500));

#[derive(Debug, Clone)]
pub struct GridMesh2D {
    graph: GraphMap<GridNode, (), petgraph::Undirected>,
}

impl Default for GridMesh2D {
    fn default() -> Self {
        Self::new()
    }
}

impl GridMesh2D {
    pub fn graph(&self) -> &GraphMap<GridNode, (), petgraph::Undirected> {
        &self.graph
    }
    /// create a new gridmesh
    pub fn new() -> Self {
        let mut graph = GraphMap::new();
        // let ssb = ssb.inflate(1, 1);
        for i in DIM.min.x..DIM.max.x {
            for j in DIM.min.y..DIM.max.y {
                graph.add_edge(
                    GridNode(SSPoint::new(i, j)),
                    GridNode(SSPoint::new(i + 1, j)),
                    (),
                );
                graph.add_edge(
                    GridNode(SSPoint::new(i, j)),
                    GridNode(SSPoint::new(i, j + 1)),
                    (),
                );
            }
        }
        Self { graph }
    }
    /// fn to remove nodes (to be called when element placed down in circuit)
    pub fn remove_nodes(&mut self, nodes: Vec<SSPoint>) {
        for n in nodes {
            self.graph.remove_node(GridNode(n));
        }
    }
    /// fn to create nodes (to be called when element removed from circuit)
    pub fn add_nodes(&mut self, nodes: Vec<SSPoint>) {
        for gn in nodes.iter().map(|&ssp| GridNode(ssp)) {
            if !DIM.contains(gn.0) {
                continue;
            }
            self.graph.add_node(gn);
            for gnn in gn.neighbors() {
                if self.graph.contains_node(gnn) {
                    self.graph.add_edge(gn, gnn, ());
                }
            }
        }
    }
    /// fn to resize meshgrid (to be called when circuit bounding box changes)
    /// not yet in use, currently just initialize a large-ish meshgrid and hope its enough
    fn resize(&mut self, _ssb: SSBox) {
        todo!()
    }

    // imagined use:
    // member of Circuit
    // empty default
    // if circuit has any element: keep grid mesh same size as bounding box plus 100 units of border - is there a need to downsize grid?
    // keep gridmesh in sync by calling add_nodes/remove_nodes upon placing/removing circuit elements
    // add/remove nodes function should resize gridmesh if needed
}
