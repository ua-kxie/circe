//! schematic net/wires
//! handles pathfinding, self pruning

use std::collections::HashSet;
use std::rc::Rc;

use crate::transforms::{SSPoint, VCTransform, VSBox, VSPoint};
use petgraph::graphmap::GraphMap;

use crate::schematic::elements::{NetEdge, NetVertex};

use crate::Drawable;

use self::pathfinding::DijkstraSt;

mod pathfinding;
use crate::schematic::layers::nets::pathfinding::path_to_goal;
use crate::schematic::layers::nets::pathfinding::wiring_pathfinder;

mod pruning;
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

#[derive(Clone)]
pub struct Nets {
    pub graph: Box<GraphMap<NetVertex, NetEdge, petgraph::Undirected>>,
    label_manager: LabelManager,
    dijkstrast: DijkstraSt,
}

impl Default for Nets {
    fn default() -> Self {
        Nets {
            graph: Box::new(GraphMap::new()),
            label_manager: LabelManager::default(),
            dijkstrast: DijkstraSt::new(SSPoint::origin()),
        }
    }
}

impl Nets {
    pub fn new(ssp: SSPoint) -> Self {
        Nets {
            graph: Box::new(GraphMap::new()),
            label_manager: LabelManager::default(),
            dijkstrast: DijkstraSt::new(ssp),
        }
    }
    pub fn dijkstra_start(&self) -> SSPoint {
        self.dijkstrast.start()
    }
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
    /// this function is called whenever schematic is changed. Ensures all connected nets have the same net name, overlapping segments are merged, etc.
    /// extra_vertices are coordinates where net segments should be bisected (device ports)
    pub fn prune(&mut self, extra_vertices: &[SSPoint]) {
        pruning::prune(self, extra_vertices);
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
    /// returns true if any net vertex is on ssp
    pub fn any_vertex_occupy_ssp(&self, ssp: SSPoint) -> bool {
        self.graph.nodes().any(|v| v.0 == ssp)
    }
    fn basic_route(src: SSPoint, dst: SSPoint) -> Box<[SSPoint]> {
        // just force edges to be vertical or horizontal - oblique later
        let delta = dst - src;
        match (delta.x, delta.y) {
            (0, 0) => Box::new([]),
            (0, _y) => Box::new([src, dst]),
            (_x, 0) => Box::new([src, dst]),
            (_x, y) => {
                let corner = SSPoint::new(src.x, src.y + y);
                Box::new([src, corner, dst])
            }
        }
    }
    /// add net segments to connect src and dst
    pub fn route(&mut self, edge_cost: &impl Fn(SSPoint, SSPoint, SSPoint) -> f32, dst: SSPoint) {
        // run pathfinding
        let goals = Box::from([dst]);
        wiring_pathfinder(&goals, &mut self.dijkstrast, edge_cost);
        let path = path_to_goal(&self.dijkstrast, &goals);

        // use fallback incase route failed
        let path = path.unwrap_or_else(|| Self::basic_route(self.dijkstrast.start(), dst));
        if path.is_empty() {
            // if path is empty - src and dst same point
            return
        }

        // filter redundant nodes - necessary to avoid solder dots where crossing another net segment
        let mut simple_path = Vec::with_capacity(path.len());
        simple_path.push(*path.first().unwrap());
        for i in 1..path.len() - 1 {
            if (path[i - 1].x != path[i + 1].x) && (path[i - 1].y != path[i + 1].y) {
                simple_path.push(path[i]);
            }
        }
        simple_path.push(*path.last().unwrap());

        // create the path with NetEdges
        for i in 1..simple_path.len() {
            let interactable = NetEdge::interactable(simple_path[i - 1], simple_path[i]);
            self.graph.add_edge(
                NetVertex(simple_path[i - 1]),
                NetVertex(simple_path[i]),
                NetEdge {
                    src: simple_path[i - 1],
                    dst: simple_path[i],
                    interactable,
                    ..Default::default()
                },
            );
        }
    }
    /// merge other into self. extra_vertices are coordinates where net segments should be bisected (device ports)
    pub fn merge(&mut self, other: &Nets, extra_vertices: &[SSPoint]) {
        for edge in other.graph.all_edges() {
            let mut ew = edge.2.clone();
            ew.interactable = NetEdge::interactable(edge.0 .0, edge.1 .0);
            self.graph.add_edge(edge.0, edge.1, ew); // adding edges also add nodes if they do not already exist
        }
        self.prune(extra_vertices);
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
