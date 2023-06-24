//! schematic net/wires

use std::collections::HashSet;
use std::rc::Rc;

use crate::{
    schematic::{interactable::Interactive, BaseElement, SchematicSet},
    transforms::{SSBox, SSPoint, SSTransform, VCTransform},
};
use petgraph::algo::tarjan_scc;
use petgraph::graphmap::GraphMap;

mod vertex;
pub use vertex::NetVertex;

mod edge;
pub use edge::NetEdge;

use super::Drawable;

/// This struct facillitates the creation of unique net names
#[derive(Clone, Debug, Default)]
struct LabelManager {
    /// watermark for floating nets
    float_wm: usize,
    /// watermark for net names
    wm: usize,
    /// set of labels already in use
    pub labels: HashSet<Rc<String>>,
}

impl LabelManager {
    /// returns a `Rc<String>`, guaranteed to be unique from all labels already registered with LabelManager.
    /// The returned String is registered and the same net name will not be returned again.
    fn new_label(&mut self) -> Rc<String> {
        loop {
            let l = format!("net_{}", self.wm);
            self.wm += 1;
            if !self.labels.contains(&l) {
                self.labels.insert(Rc::new(l.clone()));
                break self.labels.get(&Rc::new(l)).unwrap().clone();
            }
        }
    }
    /// returns a String, guaranteed to be unique from all labels already registered with LabelManager.
    /// The returned String is not registered but will not be returned again until floating nets is reset.
    /// intended for generating unique net names for devices which port(s) is left unconnected.
    fn new_floating_label(&mut self) -> String {
        loop {
            let l = format!("fn_{}", self.float_wm);
            self.float_wm += 1;
            if !self.labels.contains(&l) {
                break l;
            }
        }
    }
    /// sets float_wm to 0. Intended to be called everytime a netlist is generated.
    /// I.e. no need for net names to be unique between multiple netlists.
    fn rst_floating_nets(&mut self) {
        self.float_wm = 0;
    }
    /// register a new label
    fn register(&mut self, label: Rc<String>) {
        self.labels.insert(label);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SchematicNetLabel {
    label: String,
    // other stuff for drawing on schematic, being edited from schematic
}
#[derive(Debug, Clone)]
pub struct Nets {
    pub graph: Box<GraphMap<NetVertex, NetEdge, petgraph::Undirected>>,
    label_manager: LabelManager,
}

impl Default for Nets {
    fn default() -> Self {
        Nets {
            graph: Box::new(GraphMap::new()),
            label_manager: LabelManager::default(),
        }
    }
}

impl Nets {
    pub fn pre_netlist(&mut self) {
        self.label_manager.rst_floating_nets();
    }
    pub fn net_at(&mut self, ssp: SSPoint) -> String {
        for e in self.graph.all_edges() {
            if e.2.interactable.contains_ssp(ssp) {
                return e.2.label.as_ref().unwrap().to_string();
            }
        }
        self.label_manager.new_floating_label()
    }
    pub fn tentatives_by_ssbox(&mut self, ssb: &SSBox) {
        for e in self.graph.all_edges_mut() {
            if e.2.interactable.bounds.intersects(ssb) {
                e.2.interactable.tentative = true;
            }
        }
    }
    pub fn tentatives(&self) -> impl Iterator<Item = NetEdge> + '_ {
        self.graph.all_edges().filter_map(|e| {
            if e.2.interactable.tentative {
                Some(e.2.clone())
            } else {
                None
            }
        })
    }
    pub fn clear(&mut self) {
        self.graph.clear();
    }
    /// given a vector of vertices, return a vector of edges between the given vertices
    fn nodes_to_edge_nodes(&self, vertices: Vec<NetVertex>) -> Vec<(NetVertex, NetVertex)> {
        let mut set = HashSet::<SSPoint>::new();
        let mut ret = vec![];
        for n in vertices {
            for e in self.graph.edges(n) {
                if !set.contains(&e.1 .0) {
                    ret.push((e.0, e.1))
                } // if dst has already been iterated through, the edge was already accounted for
            }
            set.insert(n.0); // note that the source has been iterated through
        }
        ret
    }
    /// finds an appropriate net name and assigns it to all edge in edges.
    fn unify_labels(
        &mut self,
        edges: Vec<(NetVertex, NetVertex)>,
        taken_net_names: &[Rc<String>],
    ) -> Rc<String> {
        let mut label = None;
        // get smallest untaken of existing labels, if any
        for tup in &edges {
            if let Some(ew) = self.graph.edge_weight(tup.0, tup.1) {
                if let Some(label1) = &ew.label {
                    if taken_net_names.contains(label1) {
                        continue;
                    }
                    if label.is_none() || label1 < label.as_ref().unwrap() {
                        label = Some(label1.clone());
                    }
                }
            }
        }
        // if no edge is labeled, create a new label
        if label.is_none() {
            label = Some(self.label_manager.new_label());
        }
        // assign label to all edges
        for tup in edges {
            if let Some(ew) = self.graph.edge_weight_mut(tup.0, tup.1) {
                ew.label = label.clone();
            }
        }
        label.unwrap()
    }
    pub fn prune(&mut self, extra_vertices: Vec<SSPoint>) {
        // extra vertices to add, e.g. ports
        let all_vertices: Vec<NetVertex> = self.graph.nodes().collect();
        // bisect edges
        for v in &all_vertices {
            let mut colliding_edges = vec![];
            for e in self.graph.all_edges() {
                if e.2.intersects_ssp(v.0) {
                    colliding_edges.push((e.0, e.1, e.2.label.clone()));
                }
            }
            if !colliding_edges.is_empty() {
                for e in colliding_edges {
                    self.graph.remove_edge(e.0, e.1);
                    self.graph.add_edge(
                        e.0,
                        *v,
                        NetEdge {
                            src: e.0 .0,
                            dst: v.0,
                            label: e.2.clone(),
                            interactable: NetEdge::interactable(e.0 .0, v.0, false),
                            ..Default::default()
                        },
                    );
                    self.graph.add_edge(
                        e.1,
                        *v,
                        NetEdge {
                            src: e.1 .0,
                            dst: v.0,
                            label: e.2,
                            interactable: NetEdge::interactable(e.1 .0, v.0, false),
                            ..Default::default()
                        },
                    );
                }
            }
        }
        // delete redundant vertices
        for v in all_vertices {
            let connected_vertices: Vec<NetVertex> = self.graph.neighbors(v).collect();

            match connected_vertices.len() {
                0 => {
                    self.graph.remove_node(v);
                }
                2 => {
                    let del = connected_vertices[1].0 - connected_vertices[0].0;
                    match (del.x, del.y) {
                        (0, _y) => {}
                        (_x, 0) => {}
                        _ => continue,
                    }
                    let first_e = self.graph.edges(v).next().unwrap();
                    let src = connected_vertices[0];
                    let dst = connected_vertices[1];
                    let ew = NetEdge {
                        src: src.0,
                        dst: dst.0,
                        label: first_e.2.label.clone(),
                        interactable: NetEdge::interactable(src.0, dst.0, false),
                        ..Default::default()
                    };
                    if ew.intersects_ssp(v.0) {
                        self.graph.remove_node(v);
                        self.graph.add_edge(src, dst, ew);
                    }
                }
                _ => {}
            }
        }
        // bisect edges with ports
        for v in extra_vertices {
            let mut colliding_edges = vec![];
            for e in self.graph.all_edges() {
                if e.2.intersects_ssp(v) {
                    colliding_edges.push((e.0, e.1, e.2.label.clone()));
                }
            }
            if !colliding_edges.is_empty() {
                for e in colliding_edges {
                    self.graph.remove_edge(e.0, e.1);
                    self.graph.add_edge(
                        e.0,
                        NetVertex(v),
                        NetEdge {
                            src: e.0 .0,
                            dst: v,
                            label: e.2.clone(),
                            interactable: NetEdge::interactable(e.0 .0, v, false),
                            ..Default::default()
                        },
                    );
                    self.graph.add_edge(
                        e.1,
                        NetVertex(v),
                        NetEdge {
                            src: e.1 .0,
                            dst: v,
                            label: e.2,
                            interactable: NetEdge::interactable(e.1 .0, v, false),
                            ..Default::default()
                        },
                    );
                }
            }
        }
        // assign net names
        // for each subnet
        // unify labels - give vector of taken labels
        let subgraph_vertices = tarjan_scc(&*self.graph); // this finds the subnets
        let mut taken_net_names = vec![];
        for vertices in subgraph_vertices {
            let edges = self.nodes_to_edge_nodes(vertices);
            taken_net_names.push(self.unify_labels(edges, &taken_net_names));
        }
    }
    pub fn edge_occupies_ssp(&self, ssp: SSPoint) -> bool {
        for (_, _, edge) in self.graph.all_edges() {
            if edge.interactable.contains_ssp(ssp) {
                // does not include endpoints
                return true;
            }
        }
        false
    }
    pub fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.edge_occupies_ssp(ssp)
    }
    pub fn route(&mut self, src: SSPoint, dst: SSPoint) {
        // pathfinding?
        // for now, just force edges to be vertical or horizontal
        let delta = dst - src;
        match (delta.x, delta.y) {
            (0, 0) => {}
            (0, _y) => {
                let interactable = NetEdge::interactable(src, dst, true);
                self.graph.add_edge(
                    NetVertex(src),
                    NetVertex(dst),
                    NetEdge {
                        src,
                        dst,
                        interactable,
                        ..Default::default()
                    },
                );
            }
            (_x, 0) => {
                let interactable = NetEdge::interactable(src, dst, true);
                self.graph.add_edge(
                    NetVertex(src),
                    NetVertex(dst),
                    NetEdge {
                        src,
                        dst,
                        interactable,
                        ..Default::default()
                    },
                );
            }
            (_x, y) => {
                let corner = SSPoint::new(src.x, src.y + y);
                let interactable = NetEdge::interactable(src, corner, true);
                self.graph.add_edge(
                    NetVertex(src),
                    NetVertex(corner),
                    NetEdge {
                        src,
                        dst: corner,
                        interactable,
                        ..Default::default()
                    },
                );
                let interactable = NetEdge::interactable(corner, dst, true);
                self.graph.add_edge(
                    NetVertex(corner),
                    NetVertex(dst),
                    NetEdge {
                        src: corner,
                        dst,
                        interactable,
                        ..Default::default()
                    },
                );
            }
        }
    }
    pub fn merge(&mut self, other: &Nets, extra_vertices: Vec<SSPoint>) {
        for edge in other.graph.all_edges() {
            let mut ew = edge.2.clone();
            ew.interactable = NetEdge::interactable(edge.0 .0, edge.1 .0, false);
            // ew.label = Some(self.label_manager.new_label());
            self.graph.add_edge(edge.0, edge.1, ew); // adding edges also add nodes if they do not already exist
        }
        self.prune(extra_vertices);
    }
    pub fn transform(&mut self, mut e: NetEdge, sst: SSTransform) {
        self.graph.remove_edge(NetVertex(e.src), NetVertex(e.dst));
        e.transform(sst);
        self.graph.add_edge(NetVertex(e.src), NetVertex(e.dst), e);
    }
    pub fn clear_tentatives(&mut self) {
        for e in self.graph.all_edges_mut() {
            e.2.interactable.tentative = false;
        }
    }
    pub fn delete_edge(&mut self, e: &NetEdge) {
        self.graph.remove_edge(NetVertex(e.src), NetVertex(e.dst));
    }
}

impl SchematicSet for Nets {
    fn selectable(
        &mut self,
        curpos_ssp: SSPoint,
        skip: &mut usize,
        count: &mut usize,
    ) -> Option<BaseElement> {
        for e in self.graph.all_edges_mut() {
            if e.2.interactable.contains_ssp(curpos_ssp) {
                *count += 1;
                if *count > *skip {
                    *skip = *count;
                    return Some(BaseElement::NetEdge(e.2.clone()));
                }
            }
        }
        None
    }
}

impl Drawable for Nets {
    fn draw_persistent(
        &self,
        vct: VCTransform,
        vcscale: f32,
        frame: &mut iced::widget::canvas::Frame,
    ) {
        for (_, _, edge) in self.graph.all_edges() {
            edge.draw_persistent(vct, vcscale, frame)
        }
        for vertex in self.graph.nodes() {
            vertex.draw_persistent(vct, vcscale, frame)
        }
    }

    fn draw_selected(
        &self,
        _vct: VCTransform,
        _vcscale: f32,
        _frame: &mut iced::widget::canvas::Frame,
    ) {
        panic!("not intended for use");
    }

    fn draw_preview(
        &self,
        vct: VCTransform,
        vcscale: f32,
        frame: &mut iced::widget::canvas::Frame,
    ) {
        for (_, _, edge) in self
            .graph
            .all_edges()
            .filter(|e| e.2.interactable.tentative)
        {
            edge.draw_preview(vct, vcscale, frame)
        }
    }
}
