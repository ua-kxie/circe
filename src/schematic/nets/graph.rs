use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

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

type Label = Rc<RefCell<String>>;

#[derive(Clone, Debug, Default, PartialOrd, PartialEq, Eq, Hash)]
struct Label0 {
    name: String,
}

impl Label0 {
    fn new_with_ord(ord: usize) -> Label0 {
        Label0{name: format!("net_{}", ord)}
    }
}

#[derive(Clone, Debug, Default)]
struct LabelManager {
    wm: usize,
    labels: HashSet<Rc<String>>,
}

impl LabelManager {
    fn new_label(&mut self) -> Rc<String> {
        // create a new unique label, store it, return it
        loop {
            self.wm += 1;
            let l = format!("net_{}", self.wm);
            if !self.labels.contains(&l) {
                self.labels.insert(Rc::new(l.clone()));
                break self.labels.get(&Rc::new(l)).unwrap().clone()
            }
        }
    }
    fn get_label(&mut self, s: String) -> Rc<String> {
        self.labels.insert(Rc::new(s.clone()));
        self.labels.get(&Rc::new(s)).unwrap().clone()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SchematicNetLabel {
    label: String,
    // other stuff for drawing on schematic, being edited from schematic
} 
#[derive(Debug, Clone)]
pub struct Nets{
    pub graph: Box<GraphMap<NetVertex, NetEdge, petgraph::Undirected>>,
    label_manager: LabelManager,
}

impl Default for Nets {
    fn default() -> Self {
        Nets{
            graph: Box::new(GraphMap::new()),
            label_manager: LabelManager::default(),
        }
    }
}

impl Nets {
    pub fn clear(&mut self) {
        self.graph.clear();
    }
    pub fn prune(&mut self, extra_vertices: Vec<SSPoint>) {  // extra vertices to add, e.g. ports
        let all_vertices: Vec<NetVertex> = self.graph.nodes().collect();
        for v in &all_vertices {  // bisect edges
            let mut colliding_edges = vec![];
            for e in self.graph.all_edges() {
                if e.2.occupies_ssp(v.0) {
                    colliding_edges.push((e.0, e.1));
                }
            }
            if !colliding_edges.is_empty() {
                for e in colliding_edges {
                    self.graph.remove_edge(e.0, e.1);
                    self.graph.add_edge(e.0, *v, NetEdge{src: e.0.0, dst: v.0, label: Some(self.label_manager.new_label()), ..Default::default()});
                    self.graph.add_edge(e.1, *v, NetEdge{src: e.1.0, dst: v.0, label: Some(self.label_manager.new_label()), ..Default::default()});
                }
            }
        }
        for v in all_vertices {  // delete redundant vertices
            let mut connected_vertices = vec![];
            for e in self.graph.edges(v) {
                connected_vertices.push(if e.0 == v { e.1 } else { e.0 });
            }
            match connected_vertices.len() {
                0 => {
                    self.graph.remove_node(v);
                }
                2 => {
                    let src = connected_vertices[0];
                    let dst = connected_vertices[1];
                    let ew = NetEdge{src: src.0, dst: dst.0, label: Some(self.label_manager.new_label()), ..Default::default()};
                    if ew.occupies_ssp(v.0) {
                        self.graph.remove_node(v);
                        self.graph.add_edge(src, dst, ew);
                    }
                }
                _ => {}
            }
        }
        for v in extra_vertices {  // bisect edges with ports
            let mut colliding_edges = vec![];
            for e in self.graph.all_edges() {
                if e.2.occupies_ssp(v) {
                    colliding_edges.push((e.0, e.1));
                }
            }
            if !colliding_edges.is_empty() {
                for e in colliding_edges {
                    self.graph.remove_edge(e.0, e.1);
                    self.graph.add_edge(e.0, NetVertex(v), NetEdge{src: e.0.0, dst: v, label: Some(self.label_manager.new_label()), ..Default::default()});
                    self.graph.add_edge(e.1, NetVertex(v), NetEdge{src: e.1.0, dst: v, label: Some(self.label_manager.new_label()), ..Default::default()});
                }
            }
        }
    }
    pub fn edge_occupies_ssp(&self, ssp: SSPoint) -> bool {
        for (_, _, edge) in self.graph.all_edges() {
            if edge.occupies_ssp(ssp) {  // does not include endpoints
                return true;
            }
        }
        false
    }
    pub fn vertex_occupies_ssp(&self, ssp: SSPoint) -> bool {
        for v in self.graph.nodes() {
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
                self.graph.add_edge(NetVertex(src), NetVertex(dst), NetEdge{src, dst, tentative: true, ..Default::default()});
            },
            (_x, 0) => {
                self.graph.add_edge(NetVertex(src), NetVertex(dst), NetEdge{src, dst, tentative: true, ..Default::default()});
            },
            (_x, y) => {
                let corner = Point2D::new(src.x, src.y + y);
                self.graph.add_edge(NetVertex(src), NetVertex(corner), NetEdge{src, dst: corner, tentative: true, ..Default::default()});
                self.graph.add_edge(NetVertex(corner), NetVertex(dst), NetEdge{src: corner, dst, tentative: true, ..Default::default()});
            }
        }
    }
    pub fn merge(&mut self, other: &Nets, extra_vertices: Vec<SSPoint>) {
        for edge in other.graph.all_edges() {
            let mut ew = edge.2.clone();
            ew.label = Some(self.label_manager.new_label());
            self.graph.add_edge(edge.0, edge.1, ew);  // adding edges also add nodes if they do not already exist
        }
        self.prune(extra_vertices);
    }
    pub fn tentatives_to_selected(&mut self) {
        for e in self.graph.all_edges_mut().filter(|e| e.2.tentative) {
            e.2.selected = true;
            e.2.tentative = false;
        }
    }
    pub fn move_selected(&mut self, ssv: Vector2D<i16, SchematicSpace>) {
        let mut tmp = vec![];
        for e in self.graph.all_edges().filter(|e| e.2.selected) {
            tmp.push((e.0, e.1, e.2.label.clone()));
        }
        for e in tmp {
            self.graph.remove_edge(e.0, e.1);
            let (ssp0, ssp1) = (e.0.0 + ssv, e.1.0 + ssv);
            self.graph.add_edge(NetVertex(ssp0), NetVertex(ssp1), NetEdge{src: ssp0, dst: ssp1, label: e.2, ..Default::default()});
        }
    }
    pub fn draw_selected_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for e in self.graph.all_edges().filter(|e| e.2.selected) {
            e.2.draw_preview(vct, vcscale, frame);
        }
    }
    pub fn tt(&self) {
        let a = tarjan_scc(&*self.graph);  // this finds the unconnected components 
        dbg!(a);
    }
    pub fn clear_selected(&mut self) {
        for e in self.graph.all_edges_mut() {
            e.2.selected = false;
        }
    }
    pub fn clear_tentatives(&mut self) {
        for e in self.graph.all_edges_mut() {
            e.2.tentative = false;
        }
    }
    pub fn delete_selected_from_persistent(&mut self, extra_vertices: Vec<SSPoint>) {
        let mut tmp = vec![];
        for e in self.graph.all_edges().filter(|e| e.2.selected) {
            tmp.push((e.0, e.1));
        }
        for e in tmp {
            self.graph.remove_edge(e.0, e.1);
        }
        self.prune(extra_vertices);
    }
}

impl Drawable for Nets {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut iced::widget::canvas::Frame) {
        for (_, _, edge) in self.graph.all_edges() {
            edge.draw_persistent(vct, vcscale, frame)
        }
        for vertex in self.graph.nodes() {
            vertex.draw_persistent(vct, vcscale, frame)
        }
    }

    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut iced::widget::canvas::Frame) {
        for (_, _, edge) in self.graph.all_edges() {
            if edge.selected {
                edge.draw_selected(vct, vcscale, frame)
            }
        }
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut iced::widget::canvas::Frame) {
        for (_, _, edge) in self.graph.all_edges().filter(|e| e.2.tentative) {
            edge.draw_preview(vct, vcscale, frame)
        }
    }
}