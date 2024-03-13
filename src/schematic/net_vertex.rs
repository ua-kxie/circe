//! NetVertex, also used as keys to petgraph::GraphMap

use std::cmp::Ordering;

use crate::types::SSPoint;

/// petgraph vertices weight.
/// In GraphMap, also serve as the keys.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NetVertex(pub SSPoint);

/// two vertices are equal if their coordinates are equal
impl PartialOrd for NetVertex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// ord of points based on their x - y coordinate tuple
impl Ord for NetVertex {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0.x, self.0.y).cmp(&(other.0.x, other.0.y))
    }
}
