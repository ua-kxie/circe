//! schematic net/wires

use std::collections::HashSet;
use std::rc::Rc;

use crate::{
    schematic::interactable::Interactive,
    transforms::{SSBox, SSPoint, VCTransform, VSBox, VSPoint, VVTransform},
};
use petgraph::graphmap::GraphMap;
use petgraph::{algo::tarjan_scc, visit::EdgeRef};

use crate::schematic::elements::{NetEdge, NetVertex};

use crate::Drawable;

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
    #[allow(dead_code)]
    fn register(&mut self, label: Rc<String>) {
        self.labels.insert(label);
    }
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
    pub fn new_floating_label(&mut self) -> String {
        self.label_manager.new_floating_label()
    }
    /// returns the first NetEdge after skip which intersects with curpos_ssp in a BaseElement, if any.
    /// count is updated to track the number of elements skipped over
    pub fn selectable(
        &mut self,
        curpos_vsp: VSPoint,
        skip: usize,
        count: &mut usize,
    ) -> Option<NetEdge> {
        for e in self.graph.all_edges_mut() {
            if e.2.interactable.contains_vsp(curpos_vsp) {
                if *count == skip {
                    // has skipped enough elements
                    return Some(e.2.clone());
                } else {
                    *count += 1;
                }
            }
        }
        None
    }
    pub fn bounding_box(&self) -> crate::transforms::VSBox {
        VSBox::from_points(self.graph.nodes().map(|x| x.0.cast().cast_unit()))
    }
    /// this function is called before netlisting
    pub fn pre_netlist(&mut self) {
        // reset floating nets - new netlists generated with same floating net names, no need for floating net names to be unique across different netlists
        self.label_manager.rst_floating_nets();
    }
    /// returns the netname at coordinate ssp. If no net at ssp, returns a unique net name not used anywhere else (floating net)
    pub fn net_name_at(&mut self, ssp: SSPoint) -> Option<String> {
        for e in self.graph.all_edges() {
            if e.2.interactable.contains_ssp(ssp) {
                return Some(e.2.label.as_ref().unwrap().to_string());
            }
        }
        None
        // self.label_manager.new_floating_label()
    }
    /// return unique NetEdges intersecting with vsb
    pub fn intersects_vsbox(&mut self, vsb: &VSBox) -> Vec<NetEdge> {
        let mut ret = vec![];
        for e in self.graph.all_edges() {
            if e.2.interactable.bounds.intersects(vsb) {
                ret.push(e.2.clone());
            }
        }
        ret
    }
    /// return unique NetEdges bound within vsb
    pub fn contained_by(&mut self, vsb: &VSBox) -> Vec<NetEdge> {
        let mut ret = vec![];
        for e in self.graph.all_edges() {
            if vsb.contains_box(&e.2.interactable.bounds) {
                ret.push(e.2.clone());
            }
        }
        ret
    }
    /// delete all nets
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
    /// this function is called whenever schematic is changed. Ensures all connected nets have the same net name, overlapping segments are merged, etc.
    /// extra_vertices are coordinates where net segments should be bisected (device ports)
    pub fn prune(&mut self, extra_vertices: Vec<SSPoint>) {
        // extra vertices to add, e.g. ports
        let all_vertices: Vec<NetVertex> = self.graph.nodes().collect();
        // bisect edges
        for v in &all_vertices {
            let mut colliding_edges = vec![];
            for e in self.graph.all_edges() {
                if e.2.intersects_ssp(v.0.cast().cast_unit()) {
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
                            interactable: NetEdge::interactable(e.0 .0, v.0),
                        },
                    );
                    self.graph.add_edge(
                        e.1,
                        *v,
                        NetEdge {
                            src: e.1 .0,
                            dst: v.0,
                            label: e.2,
                            interactable: NetEdge::interactable(e.1 .0, v.0),
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
                    let delta = connected_vertices[1].0 - connected_vertices[0].0;
                    match (delta.x, delta.y) {
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
                        interactable: NetEdge::interactable(src.0, dst.0),
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
                            interactable: NetEdge::interactable(e.0 .0, v),
                        },
                    );
                    self.graph.add_edge(
                        e.1,
                        NetVertex(v),
                        NetEdge {
                            src: e.1 .0,
                            dst: v,
                            label: e.2,
                            interactable: NetEdge::interactable(e.1 .0, v),
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
    /// returns true if any net segment intersects with ssp
    pub fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        for (_, _, edge) in self.graph.all_edges() {
            if edge.interactable.contains_vsp(ssp.cast().cast_unit()) {
                return true;
            }
        }
        false
    }
    fn basic_route(src: SSPoint, dst: SSPoint) -> Vec<NetVertex> {
        // just force edges to be vertical or horizontal
        let delta = dst - src;
        match (delta.x, delta.y) {
            (0, 0) => {
                vec![]
            }
            (0, _y) => {
                vec![NetVertex(src), NetVertex(dst)]
            }
            (_x, 0) => {
                vec![NetVertex(src), NetVertex(dst)]
            }
            (_x, y) => {
                let corner = SSPoint::new(src.x, src.y + y);
                vec![NetVertex(src), NetVertex(corner), NetVertex(dst)]
            }
        }
    }
    /// add net segments to connect src and dst
    pub fn route(
        &mut self,
        is_occupied: &impl Fn(SSPoint) -> bool,
        src: SSPoint,
        dst: SSPoint,
        ssb: SSBox,
    ) {
        fn xy_connect(
            ssp: SSPoint,
            ssb: SSBox,
            map: &mut GraphMap<NetVertex, PathWeight, petgraph::Undirected>,
            is_occupied: &impl Fn(SSPoint) -> bool,
        ) {
            for x in (ssp.x + 1)..=ssb.max.x {
                if is_occupied(SSPoint::new(x, ssp.y)) {
                    break;
                } else {
                    map.add_edge(
                        NetVertex(ssp),
                        NetVertex(SSPoint::new(x, ssp.y)),
                        PathWeight {
                            cost: f32::ln((x - ssp.x + 1) as f32),
                        },
                    );
                }
            }
            for y in (ssp.y + 1)..=ssb.max.y {
                if is_occupied(SSPoint::new(ssp.x, y)) {
                    break;
                } else {
                    map.add_edge(
                        NetVertex(ssp),
                        NetVertex(SSPoint::new(ssp.x, y)),
                        PathWeight {
                            cost: f32::ln((y - ssp.y + 1) as f32),
                        },
                    );
                }
            }
        }
        use crate::schematic::elements::PathWeight;
        // create graph to conduct pathfinding with
        let mut m: GraphMap<NetVertex, PathWeight, petgraph::Undirected> = GraphMap::new();
        for i in ssb.min.x..=ssb.max.x {
            for j in ssb.min.y..=ssb.max.y {
                // for every point in box:
                if is_occupied(SSPoint::new(i, j)) {
                    continue;
                }
                xy_connect(SSPoint::new(i, j), ssb, &mut m, is_occupied);
            }
        }
        let path = petgraph::algo::astar(
            &m,
            NetVertex(src),
            |fin| fin.0 == dst,
            |e| e.weight().cost,
            |v| ((v.0 - dst).x.abs() + (v.0 - dst).y.abs()) as f32,
        );
        let path = path
            .or_else(|| Some((0.0, Self::basic_route(src, dst))))
            .unwrap()
            .1;

        for i in 1..path.len() {
            let interactable = NetEdge::interactable(path[i - 1].0, path[i].0);
            self.graph.add_edge(
                path[i - 1],
                path[i],
                NetEdge {
                    src: path[i - 1].0,
                    dst: path[i].0,
                    interactable,
                    ..Default::default()
                },
            );
        }
    }
    /// merge other into self. extra_vertices are coordinates where net segments should be bisected (device ports)
    pub fn merge(&mut self, other: &Nets, extra_vertices: Vec<SSPoint>) {
        for edge in other.graph.all_edges() {
            let mut ew = edge.2.clone();
            ew.interactable = NetEdge::interactable(edge.0 .0, edge.1 .0);
            self.graph.add_edge(edge.0, edge.1, ew); // adding edges also add nodes if they do not already exist
        }
        self.prune(extra_vertices);
    }
    /// applies transformation sst to NetEdge e. Moves an existing edge or adds a new one with the transformation applied.
    pub fn transform(&mut self, mut e: NetEdge, sst: VVTransform) {
        self.graph.remove_edge(NetVertex(e.src), NetVertex(e.dst));
        e.transform(sst);
        self.graph.add_edge(NetVertex(e.src), NetVertex(e.dst), e);
    }
    /// deletes NetEdge e from self
    pub fn delete_edge(&mut self, e: &NetEdge) {
        self.graph.remove_edge(NetVertex(e.src), NetVertex(e.dst));
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
        for (_, _, edge) in self.graph.all_edges() {
            edge.draw_preview(vct, vcscale, frame);
        }
    }
}
