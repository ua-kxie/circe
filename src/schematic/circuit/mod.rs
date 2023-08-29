//! Circuit
//! Concrete types for schematic content

use crate::schematic::elements::RcRDevice;
use crate::schematic::elements::RcRLabel;
use crate::schematic::elements::{NetEdge, NetVertex};
use crate::schematic::layers::Devices;
use crate::schematic::layers::NetLabels;
use crate::schematic::layers::Nets;
use crate::schematic::models::NgModels;
use crate::schematic::{self, interactable::Interactive, SchematicElement, SchematicMsg};
use crate::transforms::VSPoint;
use crate::transforms::{SSPoint, VCTransform, VSBox, VVTransform};
use crate::Drawable;
use iced::keyboard::Modifiers;
use iced::widget::canvas::{event::Event, Frame};
use paprika::PkVecvaluesall;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::rc::Rc;

use std::{collections::HashSet, fs};

mod gui;
pub use gui::CircuitPageMsg;
pub use gui::CircuitSchematicPage;

/// trait for a type of element in schematic. e.g. nets or devices
pub trait SchematicSet {
    /// returns the first element after skip which intersects with curpos_ssp in a BaseElement, if any.
    /// count is incremented by 1 for every element skipped over
    /// skip is updated if an element is returned, equal to count
    fn selectable(
        &mut self,
        curpos_ssp: SSPoint,
        skip: &mut usize,
        count: &mut usize,
    ) -> Option<CircuitElement>;

    /// returns the bounding box of all contained elements
    fn bounding_box(&self) -> VSBox;
}

/// an enum to unify different types in schematic (nets and devices)
#[derive(Debug, Clone)]
pub enum CircuitElement {
    NetEdge(NetEdge),
    Device(RcRDevice),
    Label(RcRLabel),
}

impl PartialEq for CircuitElement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NetEdge(l0), Self::NetEdge(r0)) => *l0 == *r0,
            (Self::Device(l0), Self::Device(r0)) => {
                by_address::ByAddress(l0) == by_address::ByAddress(r0)
            }
            (Self::Label(l0), Self::Label(r0)) => {
                by_address::ByAddress(l0) == by_address::ByAddress(r0)
            }
            _ => false,
        }
    }
}

impl Eq for CircuitElement {}

impl std::hash::Hash for CircuitElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            CircuitElement::NetEdge(e) => e.hash(state),
            CircuitElement::Device(d) => by_address::ByAddress(d).hash(state),
            CircuitElement::Label(l) => by_address::ByAddress(l).hash(state),
        }
    }
}

impl Drawable for CircuitElement {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            CircuitElement::NetEdge(e) => e.draw_persistent(vct, vcscale, frame),
            CircuitElement::Device(d) => d.draw_persistent(vct, vcscale, frame),
            CircuitElement::Label(l) => l.draw_persistent(vct, vcscale, frame),
        }
    }

    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            CircuitElement::NetEdge(e) => e.draw_selected(vct, vcscale, frame),
            CircuitElement::Device(d) => d.draw_selected(vct, vcscale, frame),
            CircuitElement::Label(l) => l.draw_selected(vct, vcscale, frame),
        }
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            CircuitElement::NetEdge(e) => e.draw_preview(vct, vcscale, frame),
            CircuitElement::Device(d) => d.draw_preview(vct, vcscale, frame),
            CircuitElement::Label(l) => l.draw_preview(vct, vcscale, frame),
        }
    }
}

impl SchematicElement for CircuitElement {
    fn contains_vsp(&self, vsp: VSPoint) -> bool {
        match self {
            CircuitElement::NetEdge(e) => e.interactable.contains_vsp(vsp),
            CircuitElement::Device(d) => d.0.borrow().interactable.contains_vsp(vsp),
            CircuitElement::Label(l) => l.0.borrow().interactable.contains_vsp(vsp),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    CanvasEvent(Event),
    Wire,
    NetList,
    DcOp(PkVecvaluesall),
    Ac(PkVecvaluesall),
}

impl schematic::ContentMsg for Msg {
    fn canvas_event_msg(event: Event) -> Self {
        Msg::CanvasEvent(event)
    }
}

#[derive(Clone, Default)]
pub enum CircuitSt {
    #[default]
    Idle,
    Wiring(Option<Box<Nets>>),
}

/// struct holding schematic state (nets, devices, and their locations)
#[derive(Default, Clone)]
pub struct Circuit {
    pub infobarstr: Option<String>,

    state: CircuitSt,

    nets: Nets,
    devices: Devices,
    labels: NetLabels,

    curpos_ssp: SSPoint,

    device_models: NgModels,
}

impl Circuit {
    pub fn curpos_ssp(&self) -> SSPoint {
        self.curpos_ssp
    }
    fn update_cursor_vsp(&mut self, curpos_vsp: VSPoint) {
        self.curpos_ssp = curpos_vsp.round().cast().cast_unit();
        self.infobarstr = self.nets.net_name_at(self.curpos_ssp);
        match &mut self.state {
            CircuitSt::Wiring(Some(nets)) => {
                nets.clear();
                nets.route(
                    &|prev, this, next| {
                        // do not go over ports at any cost
                        // do not go over NetVertex at any cost
                        // do not go over NetLabel at any cost
                        // do not make turn over NetEdge at any cost
                        // going straight cost 1
                        // going over symbol cost 10
                        // making turn cost 30
                        if self.devices.any_port_occupy_ssp(next) {
                            // do not go over ports at any cost
                            return f32::INFINITY;
                        }
                        if self.nets.any_vertex_occupy_ssp(next) {
                            // do not go over NetVertex at any cost
                            return f32::INFINITY;
                        }
                        if self.labels.any_occupy_ssp(next) {
                            // do not go over NetLabel at any cost
                            return f32::INFINITY;
                        }
                        let is_turn = (prev.x != next.x) && (prev.y != next.y);
                        if is_turn && self.nets.occupies_ssp(this) {
                            // next point is electrically occupied - do not use
                            return f32::INFINITY;
                        }
                        let mut ret = 1.0;
                        if self.devices.occupies_ssp(next) {
                            // going through a device's graphical symbol
                            ret += 10.0;
                        }
                        if is_turn {
                            ret += 30.0;
                        }
                        ret
                    },
                    self.curpos_ssp,
                );
            }
            CircuitSt::Idle => {}
            _ => {}
        }
    }

    // returns true if the coordinate is electrically significant
    fn electrically_occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.nets.occupies_ssp(ssp) || self.devices.any_port_occupy_ssp(ssp)
    }
}

impl Drawable for Circuit {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.nets.draw_persistent(vct, vcscale, frame);
        self.devices.draw_persistent(vct, vcscale, frame);
        self.labels.draw_persistent(vct, vcscale, frame);
    }

    fn draw_selected(&self, _vct: VCTransform, _vcscale: f32, _frame: &mut Frame) {
        panic!("not intended for use");
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match &self.state {
            CircuitSt::Wiring(Some(nets)) => {
                nets.draw_preview(vct, vcscale, frame);
            }
            CircuitSt::Idle => {}
            _ => {}
        }
    }
}

impl schematic::Content<CircuitElement, Msg> for Circuit {
    fn curpos_update(&mut self, vsp: VSPoint) {
        self.update_cursor_vsp(vsp);
    }
    fn curpos_vsp(&self) -> VSPoint {
        self.curpos_ssp.cast().cast_unit()
    }
    fn bounds(&self) -> VSBox {
        let bbn = self.nets.bounding_box();
        let bbi = self.devices.bounding_box();
        let bbl = self.labels.bounding_box();
        bbn.union(&bbi).union(&bbl)
    }
    fn intersects_vsb(&mut self, vsb: VSBox) -> HashSet<CircuitElement> {
        let mut ret = HashSet::new();
        for seg in self.nets.intersects_vsbox(&vsb) {
            ret.insert(CircuitElement::NetEdge(seg));
        }
        for rcrd in self.devices.intersects_vsb(&vsb) {
            ret.insert(CircuitElement::Device(rcrd));
        }
        for rcrl in self.labels.intersects_vsb(&vsb) {
            ret.insert(CircuitElement::Label(rcrl));
        }
        ret
    }
    fn contained_by(&mut self, vsb: VSBox) -> HashSet<CircuitElement> {
        let mut ret = HashSet::new();
        for seg in self.nets.contained_by(&vsb) {
            ret.insert(CircuitElement::NetEdge(seg));
        }
        for rcrd in self.devices.contained_by(&vsb) {
            ret.insert(CircuitElement::Device(rcrd));
        }
        for rcrl in self.labels.contained_by(&vsb) {
            ret.insert(CircuitElement::Label(rcrl));
        }
        ret
    }

    /// returns the first CircuitElement after skip which intersects with curpos_ssp, if any.
    /// count is updated to track the number of elements skipped over
    fn selectable(
        &mut self,
        vsp: VSPoint,
        skip: usize,
        count: &mut usize,
    ) -> Option<CircuitElement> {
        if let Some(l) = self.labels.selectable(vsp, skip, count) {
            return Some(CircuitElement::Label(l));
        }
        if let Some(e) = self.nets.selectable(vsp, skip, count) {
            return Some(CircuitElement::NetEdge(e));
        }
        if let Some(d) = self.devices.selectable(vsp, skip, count) {
            return Some(CircuitElement::Device(d));
        }
        None
    }

    fn update(&mut self, msg: Msg) -> SchematicMsg<CircuitElement> {
        let ret_msg = match msg {
            Msg::CanvasEvent(event) => {
                let mut state = self.state.clone();
                let mut ret_msg_tmp = SchematicMsg::None;
                const NO_MODIFIER: Modifiers = Modifiers::empty();
                match (&mut state, event) {
                    // wiring
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::W,
                            modifiers: NO_MODIFIER,
                        }),
                    ) => {
                        state = CircuitSt::Wiring(None);
                    }
                    (
                        CircuitSt::Wiring(opt_ws),
                        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
                    ) => {
                        let ssp = self.curpos_ssp();
                        let new_ws;
                        if let Some(g) = opt_ws {
                            // subsequent click
                            if ssp == g.dijkstra_start() {
                                new_ws = None;
                            } else if self.electrically_occupies_ssp(ssp) {
                                self.nets.merge(g.as_ref(), &self.devices.ports_ssp());
                                new_ws = None;
                            } else {
                                self.nets.merge(g.as_ref(), &self.devices.ports_ssp());
                                new_ws = Some(Box::new(Nets::new(ssp)));
                            }
                            ret_msg_tmp = SchematicMsg::ClearPassive;
                        } else {
                            // first click
                            new_ws = Some(Box::new(Nets::new(ssp)));
                        }
                        state = CircuitSt::Wiring(new_ws);
                    }
                    // label
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::L,
                            modifiers: Modifiers::SHIFT,
                        }),
                    ) => {
                        let l = NetLabels::new_label();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(CircuitElement::Label(l)));
                    }
                    // device placement
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::C,
                            modifiers: NO_MODIFIER,
                        }),
                    ) => {
                        let d = self.devices.new_cap();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(CircuitElement::Device(d)));
                    }
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::L,
                            modifiers: NO_MODIFIER,
                        }),
                    ) => {
                        let d = self.devices.new_ind();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(CircuitElement::Device(d)));
                    }
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::P,
                            modifiers: NO_MODIFIER,
                        }),
                    ) => {
                        let d = self.devices.new_pmos();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(CircuitElement::Device(d)));
                    }
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::N,
                            modifiers: NO_MODIFIER,
                        }),
                    ) => {
                        let d = self.devices.new_nmos();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(CircuitElement::Device(d)));
                    }
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::R,
                            modifiers: NO_MODIFIER,
                        }),
                    ) => {
                        let d = self.devices.new_res();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(CircuitElement::Device(d)));
                    }
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::G,
                            modifiers: NO_MODIFIER,
                        }),
                    ) => {
                        let d = self.devices.new_gnd();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(CircuitElement::Device(d)));
                    }
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::V,
                            modifiers: NO_MODIFIER,
                        }),
                    ) => {
                        let d = self.devices.new_vs();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(CircuitElement::Device(d)));
                    }
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::I,
                            modifiers: NO_MODIFIER,
                        }),
                    ) => {
                        let d = self.devices.new_is();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(CircuitElement::Device(d)));
                    }
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::D,
                            modifiers: NO_MODIFIER,
                        }),
                    ) => {
                        let d = self.devices.new_diode();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(CircuitElement::Device(d)));
                    }
                    // state reset
                    (
                        _,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Escape,
                            modifiers: NO_MODIFIER,
                        }),
                    ) => {
                        state = CircuitSt::Idle;
                    }
                    _ => {}
                }
                self.state = state;
                ret_msg_tmp
            }
            Msg::NetList => {
                self.netlist();
                SchematicMsg::None
            }
            Msg::Wire => {
                self.state = CircuitSt::Wiring(None);
                SchematicMsg::None
            }
            Msg::DcOp(pkvecvaluesall) => {
                self.devices.op(&pkvecvaluesall);
                SchematicMsg::ClearPassive
            }
            Msg::Ac(pkvecvaluesall) => {
                self.devices.op(&pkvecvaluesall);
                SchematicMsg::ClearPassive
            }
        };
        ret_msg
    }

    fn move_elements(&mut self, elements: &mut HashSet<CircuitElement>, sst: &VVTransform) {
        let mut nets = Vec::with_capacity(elements.len());
        for e in &*elements {
            match e {
                CircuitElement::NetEdge(seg) => {
                    nets.push(seg.clone());
                }
                CircuitElement::Device(d) => {
                    d.0.borrow_mut().transform(*sst);
                    // if moving an existing device, does nothing
                    // inserts the device if placing a new device
                    self.devices.insert(d.clone());
                }
                CircuitElement::Label(l) => {
                    l.0.borrow_mut().transform(*sst);
                    // if moving an existing label, does nothing
                    // inserts the label if placing a new label
                    self.labels.insert(l.clone());
                }
            }
        }
        for n in nets {
            // remove netedge
            elements.remove(&CircuitElement::NetEdge(n.clone()));
            self.nets
                .graph
                .remove_edge(NetVertex(n.src), NetVertex(n.dst));

            // transform netedge and add
            let mut n1 = n.clone();
            n1.transform(*sst);
            elements.insert(CircuitElement::NetEdge(n1.clone()));
            self.nets
                .graph
                .add_edge(NetVertex(n1.src), NetVertex(n1.dst), n1);
        }
        self.prune();
    }

    fn copy_elements(&mut self, elements: &mut HashSet<CircuitElement>, sst: &VVTransform) {
        let vec_ce = elements.clone().into_iter().collect::<Vec<_>>();
        elements.clear(); // clear the original elements
        for ce in vec_ce {
            match ce {
                CircuitElement::NetEdge(seg) => {
                    let mut seg1 = seg.clone();
                    seg1.transform(*sst);
                    self.nets.graph.add_edge(
                        NetVertex(seg1.src),
                        NetVertex(seg1.dst),
                        seg1.clone(),
                    );
                    elements.insert(CircuitElement::NetEdge(seg1));
                }
                CircuitElement::Device(rcr) => {
                    //unwrap refcell
                    let mut device = (*rcr.0.borrow()).clone();
                    device.transform(*sst);

                    //build BaseElement
                    let d_refcell = RefCell::new(device);
                    let d_rc = Rc::new(d_refcell);
                    let rcr_device = RcRDevice(d_rc);
                    self.devices.insert(rcr_device.clone());
                    elements.insert(CircuitElement::Device(rcr_device));
                }
                CircuitElement::Label(rcl) => {
                    //unwrap refcell
                    let mut label = (*rcl.0.borrow()).clone();
                    label.transform(*sst);

                    //build BaseElement
                    let l_refcell = RefCell::new(label);
                    let l_rc = Rc::new(l_refcell);
                    let rcr_label = RcRLabel(l_rc);
                    self.labels.insert(rcr_label.clone());
                    elements.insert(CircuitElement::Label(rcr_label));
                }
            }
        }
    }

    fn delete_elements(&mut self, elements: &HashSet<CircuitElement>) {
        for e in elements {
            match e {
                CircuitElement::NetEdge(e) => {
                    self.nets.delete_edge(e);
                }
                CircuitElement::Device(d) => {
                    self.devices.delete_item(d);
                }
                CircuitElement::Label(l) => {
                    self.labels.delete_item(l);
                }
            }
        }
        self.prune();
    }

    fn is_idle(&self) -> bool {
        matches!(self.state, CircuitSt::Idle)
    }
}

impl Circuit {
    /// create netlist for the current schematic and save it.
    pub fn netlist(&mut self) {
        self.nets.pre_netlist();
        let mut netlist = String::from("Netlist Created by Circe\n");
        netlist.push_str(&self.device_models.model_definitions());
        if self.devices.get_set().is_empty() {
            // empty netlist
            netlist.push_str("V_0 0 n1 0"); // give it something so spice doesnt hang
        }
        for d in self.devices.get_set() {
            netlist.push_str(&d.0.borrow_mut().spice_line(&mut self.nets));
        }
        netlist.push('\n');
        fs::write("netlist.cir", netlist.as_bytes()).expect("Unable to write file");
    }
    /// clear up nets graph: merging segments, cleaning up segment net names, etc.
    fn prune(&mut self) {
        self.nets.prune(&self.devices.ports_ssp());
    }
}
