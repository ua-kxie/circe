use crate::transforms::{SSPoint, VCTransform};
use euclid::{Point2D};
use petgraph::graphmap::GraphMap;

mod vertex;
pub use vertex::NetVertex;

mod edge;
pub use edge::NetEdge;

use super::Drawable;
use std::cell::Cell;

#[derive(Debug, Clone)]
pub struct NetsGraph(pub GraphMap<NetVertex, NetEdge, petgraph::Undirected>);

impl Default for NetsGraph {
    fn default() -> Self {
        NetsGraph(GraphMap::new())
    }
}

pub trait NetsGraphExt {
    fn merge(&mut self, other: &NetsGraph);
    fn route(&mut self, src: SSPoint, dst: SSPoint);
    fn edge_occupies_ssp(&self, ssp: SSPoint) -> bool;
    fn vertex_occupies_ssp(&self, ssp: SSPoint) -> bool;
    fn occupies_ssp(&self, ssp: SSPoint) -> bool;
    fn clear(&mut self);
    fn prune(&mut self);
}

impl NetsGraphExt for NetsGraph {
    fn clear(&mut self) {
        self.0.clear();
    }
    fn prune(&mut self) {
        let all_vertices: Vec<NetVertex> = self.0.nodes().collect();
        for v in &all_vertices {  // bisect edges
            let mut colliding_edges = vec![];
            for e in self.0.all_edges() {
                if e.2.occupies_ssp(v.0) {
                    colliding_edges.push((e.0, e.1));
                }
            }
            if !colliding_edges.is_empty() {
                for e in colliding_edges {
                    self.0.remove_edge(e.0, e.1);
                    self.0.add_edge(e.0, *v, NetEdge{src: e.0.0, dst: v.0, ..Default::default()});
                    self.0.add_edge(e.1, *v, NetEdge{src: e.1.0, dst: v.0, ..Default::default()});
                }
            }
        }
        for v in all_vertices {  // delete redundant vertices
            let mut connected_vertices = vec![];
            for e in self.0.edges(v) {
                connected_vertices.push(if e.0 == v { e.1 } else { e.0 });
            }
            match connected_vertices.len() {
                0 => {
                    self.0.remove_node(v);
                }
                2 => {
                    let src = connected_vertices[0];
                    let dst = connected_vertices[1];
                    let ew = NetEdge{src: src.0, dst: dst.0, ..Default::default()};
                    if ew.occupies_ssp(v.0) {
                        self.0.remove_node(v);
                        self.0.add_edge(src, dst, ew);
                    }
                }
                _ => {}
            }
        }

    }
    fn edge_occupies_ssp(&self, ssp: SSPoint) -> bool {
        for (_, _, edge) in self.0.all_edges() {
            if edge.occupies_ssp(ssp) {  // does not include endpoints
                return true;
            }
        }
        false
    }
    fn vertex_occupies_ssp(&self, ssp: SSPoint) -> bool {
        for v in self.0.nodes() {
            if v.occupies_ssp(ssp) {
                return true;
            }
        }
        false
    }
    fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.vertex_occupies_ssp(ssp) || self.edge_occupies_ssp(ssp)
    }
    fn route(&mut self, src: SSPoint, dst: SSPoint) {
        // pathfinding?
        // for now, just force edges to be vertical or horizontal
        let delta = dst - src;
        match (delta.x, delta.y) {
            (0, 0) => {},
            (0, _y) => {
                self.0.add_edge(NetVertex(src), NetVertex(dst), NetEdge{src, dst, ..Default::default()});
            },
            (_x, 0) => {
                self.0.add_edge(NetVertex(src), NetVertex(dst), NetEdge{src, dst, ..Default::default()});
            },
            (_x, y) => {
                let corner = Point2D::new(src.x, src.y + y);
                self.0.add_edge(NetVertex(src), NetVertex(corner), NetEdge{src, dst: corner, ..Default::default()});
                self.0.add_edge(NetVertex(corner), NetVertex(dst), NetEdge{src: corner, dst, ..Default::default()});
            }
        }
    }
    fn merge(&mut self, other: &NetsGraph) {
        for edge in other.0.all_edges() {
            self.0.add_edge(edge.0, edge.1, edge.2.clone());  // adding edges also add nodes if they do not already exist
        }
        self.prune();
    }
}

impl Drawable for NetsGraph {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut iced::widget::canvas::Frame) {
        for (_, _, edge) in self.0.all_edges() {
            edge.draw_persistent(vct, vcscale, frame)
        }
        for vertex in self.0.nodes() {
            vertex.draw_persistent(vct, vcscale, frame)
        }
    }

    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut iced::widget::canvas::Frame) {
        for (_, _, edge) in self.0.all_edges() {
            if edge.selected {
                edge.draw_selected(vct, vcscale, frame)
            }
        }
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut iced::widget::canvas::Frame) {
        for (_, _, edge) in self.0.all_edges().filter(|e| e.2.tentative) {
            edge.draw_preview(vct, vcscale, frame)
        }
        for vertex in self.0.nodes() {
            vertex.draw_preview(vct, vcscale, frame)
        }
    }
}