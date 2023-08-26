//! Variation of Dijkstra's algo adapted for EDA wiring.
//!  
//! Allows assigning cost to making turns

use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{BinaryHeap, HashMap, HashSet};

use std::hash::Hash;

use petgraph::algo::Measure;
use petgraph::visit::{EdgeRef, IntoEdges, VisitMap, Visitable};

use std::cmp::Ordering;

pub mod grid_mesh;

/// private struct copied from petgraph
/// `MinScored<K, T>` holds a score `K` and a scored object `T` in
/// a pair for use with a `BinaryHeap`.
///
/// `MinScored` compares in reverse order by the score, so that we can
/// use `BinaryHeap` as a min-heap to extract the score-value pair with the
/// least score.
///
/// **Note:** `MinScored` implements a total order (`Ord`), so that it is
/// possible to use float types as scores.
#[derive(Copy, Clone, Debug)]
pub struct MinScored<K, T>(pub K, pub T);

impl<K: PartialOrd, T> PartialEq for MinScored<K, T> {
    #[inline]
    fn eq(&self, other: &MinScored<K, T>) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<K: PartialOrd, T> Eq for MinScored<K, T> {}

impl<K: PartialOrd, T> PartialOrd for MinScored<K, T> {
    #[inline]
    fn partial_cmp(&self, other: &MinScored<K, T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: PartialOrd, T> Ord for MinScored<K, T> {
    #[inline]
    fn cmp(&self, other: &MinScored<K, T>) -> Ordering {
        let a = &self.0;
        let b = &other.0;
        if a == b {
            Ordering::Equal
        } else if a < b {
            Ordering::Greater
        } else if a > b {
            Ordering::Less
        } else if a.ne(a) && b.ne(b) {
            // these are the NaN cases
            Ordering::Equal
        } else if a.ne(a) {
            // Order NaN less, so that it is last in the MinScore order
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

/// Modified Dijkstra specialized for schematic wiring
pub fn wiring_pathfinder<G, F, K>(
    graph: G,
    goals: &Box<[G::NodeId]>,
    mut st: DijkstraSt<G, K>,
    mut edge_cost: F,
) -> DijkstraSt<G, K>
where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash,
    // closure to get cost from parent, current, and next node
    F: FnMut(G::NodeId, G::NodeId, G::NodeId) -> K,
    K: Measure + Copy,
{
    // first check if given st already includes path to goal
    if goals.iter().any(|n| st.cost_map.contains_key(n)) {
        return st;
    }

    // start visiting frontier nodes
    while let Some(MinScored(cost, node)) = st.to_visit.pop() {
        if st.visited.is_visited(&node) {
            // was already visited through a lower cost path
            continue;
        }
        if goals.iter().any(|n| n == &node) {
            // goal was reached
            break;
        }
        let prev = st.cost_map.get(&node).unwrap().1;
        for edge in graph.edges(node) {
            let next = edge.target();
            if st.visited.is_visited(&next) {
                // already found a lower cost path to this target
                continue;
            }
            let next_score = cost + edge_cost(prev, node, next);
            match st.cost_map.entry(next) {
                Occupied(value) => {
                    if next_score < value.get().0 {
                        *value.into_mut() = (next_score, node);
                        st.to_visit.push(MinScored(next_score, next));
                    }
                }
                Vacant(value) => {
                    value.insert((next_score, node));
                    st.to_visit.push(MinScored(next_score, next));
                }
            }
        }
        st.visited.visit(node);
    }
    st
}

/// build path to target from pathfinding state
#[allow(clippy::if_same_then_else)] // to avoid panic in case of None
pub fn path_to_goal<G, K>(
    st: DijkstraSt<G, K>,
    goals: &Box<[G::NodeId]>,
) -> Option<(K, Vec<G::NodeId>)>
where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash,
    K: Measure + Copy,
{
    // find cheapest goal to reach
    let max = goals
        .iter()
        .map(|n| st.cost_map.get(n).map(|tup| MinScored(tup.0, n)))
        .max();
    if max.is_none() {
        return None;
    } else if max.unwrap().is_none() {
        return None;
    }
    let cost = max.unwrap().unwrap().0;
    let goal = max.unwrap().unwrap().1;

    // build path to reach goal
    let mut ret = vec![*goal];
    let mut breadcrumbs = HashSet::new();
    breadcrumbs.insert(*goal);
    let mut this = goal;
    loop {
        if let Some((_, prev)) = st.cost_map.get(this) {
            ret.push(*prev);
            if !breadcrumbs.insert(*prev) {
                // cyclic path detected
                return None;
            }
            if *prev == st.start {
                // backtraced to start
                break;
            }
            this = prev;
        } else {
            // return None in anything goes wrong
            return None;
        }
    }
    Some((cost, ret))
}

pub struct DijkstraSt<G, K>
where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash,
    K: Measure + Copy,
{
    // cost map of nodeid to cost and optimal parent
    cost_map: HashMap<G::NodeId, (K, G::NodeId)>,
    visited: <G as Visitable>::Map,
    to_visit: BinaryHeap<MinScored<K, G::NodeId>>,
    start: G::NodeId,
}

impl<G, K> DijkstraSt<G, K>
where
    G: IntoEdges + Visitable,
    G::NodeId: Eq + Hash,
    K: Measure + Copy,
{
    pub fn new(g: &G, start: G::NodeId) -> Self {
        let mut cost_map = HashMap::default();
        let mut to_visit = BinaryHeap::new();

        let zerocost = K::default();
        cost_map.insert(start, (zerocost, start));
        to_visit.push(MinScored(zerocost, start));
        Self {
            cost_map,
            visited: G::visit_map(g),
            to_visit,
            start,
        }
    }
}
