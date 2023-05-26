use crate::transforms::{SSPoint, VCTransform, SchematicSpace};
use euclid::{Point2D, Vector2D};
use iced::widget::canvas::Frame;
use petgraph::graphmap::GraphMap;
use petgraph::algo::tarjan_scc;

mod vertex;
pub use vertex::NetVertex;

mod edge;
pub use edge::NetEdge;

use super::Drawable;

#[derive(Debug, Clone)]
pub struct Nets(pub Box<GraphMap<NetVertex, NetEdge, petgraph::Undirected>>);

impl Default for Nets {
    fn default() -> Self {
        Nets(Box::new(GraphMap::new()))
    }
}

impl Nets {
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn prune(&mut self, extra_vertices: Vec<SSPoint>) {  // extra vertices to add, e.g. ports
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
        for v in extra_vertices {  // bisect edges
            let mut colliding_edges = vec![];
            for e in self.0.all_edges() {
                if e.2.occupies_ssp(v) {
                    colliding_edges.push((e.0, e.1));
                }
            }
            if !colliding_edges.is_empty() {
                for e in colliding_edges {
                    self.0.remove_edge(e.0, e.1);
                    self.0.add_edge(e.0, NetVertex(v), NetEdge{src: e.0.0, dst: v, ..Default::default()});
                    self.0.add_edge(e.1, NetVertex(v), NetEdge{src: e.1.0, dst: v, ..Default::default()});
                }
            }
        }
    }
    pub fn edge_occupies_ssp(&self, ssp: SSPoint) -> bool {
        for (_, _, edge) in self.0.all_edges() {
            if edge.occupies_ssp(ssp) {  // does not include endpoints
                return true;
            }
        }
        false
    }
    pub fn vertex_occupies_ssp(&self, ssp: SSPoint) -> bool {
        for v in self.0.nodes() {
            if v.occupies_ssp(ssp) {
                return true;
            }
        }
        false
    }
    pub fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.vertex_occupies_ssp(ssp) || self.edge_occupies_ssp(ssp)
    }
    pub fn route(&mut self, src: SSPoint, dst: SSPoint) {
        // pathfinding?
        // for now, just force edges to be vertical or horizontal
        let delta = dst - src;
        match (delta.x, delta.y) {
            (0, 0) => {},
            (0, _y) => {
                self.0.add_edge(NetVertex(src), NetVertex(dst), NetEdge{src, dst, tentative: true, ..Default::default()});
            },
            (_x, 0) => {
                self.0.add_edge(NetVertex(src), NetVertex(dst), NetEdge{src, dst, tentative: true, ..Default::default()});
            },
            (_x, y) => {
                let corner = Point2D::new(src.x, src.y + y);
                self.0.add_edge(NetVertex(src), NetVertex(corner), NetEdge{src, dst: corner, tentative: true, ..Default::default()});
                self.0.add_edge(NetVertex(corner), NetVertex(dst), NetEdge{src: corner, dst, tentative: true, ..Default::default()});
            }
        }
    }
    pub fn merge(&mut self, other: &Nets, extra_vertices: Vec<SSPoint>) {
        for edge in other.0.all_edges() {
            self.0.add_edge(edge.0, edge.1, edge.2.clone());  // adding edges also add nodes if they do not already exist
        }
        self.prune(extra_vertices);
    }
    pub fn tentatives_to_selected(&mut self) {
        for e in self.0.all_edges_mut().filter(|e| e.2.tentative) {
            e.2.selected = true;
            e.2.tentative = false;
        }
    }
    pub fn move_selected(&mut self, ssv: Vector2D<i16, SchematicSpace>) {
        let mut tmp = vec![];
        for e in self.0.all_edges().filter(|e| e.2.selected) {
            tmp.push((e.0, e.1));
        }
        for e in tmp {
            self.0.remove_edge(e.0, e.1);
            let (ssp0, ssp1) = (e.0.0 + ssv, e.1.0 + ssv);
            self.0.add_edge(NetVertex(ssp0), NetVertex(ssp1), NetEdge{src: ssp0, dst: ssp1, ..Default::default()});
        }
    }
    pub fn draw_selected_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for e in self.0.all_edges().filter(|e| e.2.selected) {
            e.2.draw_preview(vct, vcscale, frame);
        }
    }
    pub fn tt(&self) {
        let a = tarjan_scc(&*self.0);  // this finds the unconnected components 
    }
    pub fn clear_selected(&mut self) {
        for e in self.0.all_edges_mut() {
            e.2.selected = false;
        }
    }
    pub fn clear_tentatives(&mut self) {
        for e in self.0.all_edges_mut() {
            e.2.tentative = false;
        }
    }
    pub fn delete_selected_from_persistent(&mut self, extra_vertices: Vec<SSPoint>) {
        let mut tmp = vec![];
        for e in self.0.all_edges().filter(|e| e.2.selected) {
            tmp.push((e.0, e.1));
        }
        for e in tmp {
            self.0.remove_edge(e.0, e.1);
        }
        self.prune(extra_vertices);
    }
}

impl Drawable for Nets {
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
    }
}