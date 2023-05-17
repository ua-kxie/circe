use std::cmp::Ordering;

use euclid::Point2D;
use petgraph::{graphmap::GraphMap, algo::tarjan_scc};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewportSpace;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasSpace;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchematicSpace;

pub type VSPoint = euclid::Point2D<f32, ViewportSpace>;
pub type CSPoint = euclid::Point2D<f32, CanvasSpace>;
pub type SSPoint = euclid::Point2D<i16, SchematicSpace>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
struct SSPointOrd (SSPoint);

impl PartialOrd for SSPointOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SSPointOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0.x, self.0.y).cmp(&(other.0.x, other.0.y))
    }
}

// type NetsGraph = GraphMap<SSPointOrd, (), petgraph::Undirected>;
pub struct NetsGraph(GraphMap<SSPointOrd, (), petgraph::Undirected>);

trait NetsGraphExt {
    fn merge(&mut self, other: &NetsGraph);
    fn route(&mut self, src: SSPoint, dst: SSPoint);
    fn intersects_pt(&self, ssp: SSPoint) -> bool;
}

impl NetsGraphExt for NetsGraph {
    fn intersects_pt(&self, ssp: SSPoint) -> bool {
        for (src, dst, _) in self.0.all_edges() {
            let delta = src.0 - dst.0;
            match (delta.x, delta.y) {
                (0, 0) => {panic!("unreachable code")},
                (0, _) => {  // vertical
                    if (src.0.x == ssp.x) && (src.0.y.max(dst.0.y) >= ssp.y) && (src.0.y.min(dst.0.y) <= ssp.y) {
                        return true
                    }
                }
                (_, 0) => {  // horizontal
                    if (src.0.y == ssp.y) && (src.0.x.max(dst.0.x) >= ssp.x) && (src.0.x.min(dst.0.x) <= ssp.x) {
                        return true
                    }
                }
                _ => {}  // oblique
            }
        }
        false
    }
    fn route(&mut self, src: SSPoint, dst: SSPoint) {
        // pathfinding?
        // for now, just force edges to be vertical or horizontal
        let delta = dst - src;
        match (delta.x, delta.y) {
            (0, 0) => {},
            (0, y) => {
                self.0.add_edge(SSPointOrd(src), SSPointOrd(dst), ());
            },
            (x, 0) => {
                self.0.add_edge(SSPointOrd(src), SSPointOrd(dst), ());
            },
            (x, y) => {
                let corner = Point2D::new(src.x, src.y + y);
                self.0.add_edge(SSPointOrd(src), SSPointOrd(corner), ());
                self.0.add_edge(SSPointOrd(corner), SSPointOrd(dst), ());
            }
        }
    }
    fn merge(&mut self, other: &NetsGraph) {
        for edge in other.0.all_edges() {
            self.0.add_edge(edge.0, edge.1, ());  // adding edges also add nodes if they do not already exist
        }
    }
}

pub enum SchematicState {
    Wiring(Option<(NetsGraph, SSPoint)>),
}

struct Nets {
    persistent: NetsGraph,
    selected: NetsGraph,
}

impl Nets {
    fn event_exit(&mut self) {
        self.selected.0.clear();
    }
    fn event_confirm() {

    }
    fn tt(&self) {
        let a = tarjan_scc(&self.persistent.0);  // this finds the unconnected components 
    }
    fn new_vertex(&mut self, preview: &mut Option<(NetsGraph, SSPoint)>, ssp: SSPoint) {  // assume we are in wiring mode
        if let Some((preview_graph, prev_point)) = preview {  // subsequent point
            // if clicked on prev_point: terminate
            // if clicked on space occupied by persistent graph: terminate
            if (*prev_point == ssp) || (self.persistent.intersects_pt(ssp)) {
                *preview = None;
            } else {  // otherwise, route edges, merge into persistent, and clear preview
                preview_graph.route(*prev_point, ssp);
                self.persistent.merge(preview_graph);
                preview_graph.0.clear();
                *prev_point = ssp;
            }
        } else {  // first point
            let preview_graph = NetsGraph(GraphMap::new());
            *preview = Some((preview_graph, ssp))
        }
    }
}

fn main() {
    let mut deps = GraphMap::<SSPointOrd, (), petgraph::Undirected>::new();
}
