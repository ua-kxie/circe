//! Circuit
//! Concrete types for schematic content

use crate::schematic::devices::Devices;
use crate::schematic::nets::{NetEdge, NetVertex, Nets};
use crate::schematic::{self, RcRDevice, SchematicElement, SchematicMsg};
use crate::{interactable, viewport};
use crate::{
    interactable::Interactive,
    transforms::{
        self, CSPoint, Point, SSBox, SSPoint, SSTransform, SSVec, VCTransform, VSBox, ViewportSpace,
    },
    viewport::Drawable,
};
use iced::{
    mouse,
    widget::canvas::{self, event::Event, path::Builder, Frame, LineCap, Stroke},
    Color, Size,
};
use paprika::PkVecvaluesall;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::HashSet, fs};

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
}

impl PartialEq for CircuitElement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NetEdge(l0), Self::NetEdge(r0)) => *l0 == *r0,
            (Self::Device(l0), Self::Device(r0)) => {
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
        }
    }
}

impl Drawable for CircuitElement {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            CircuitElement::NetEdge(e) => e.draw_persistent(vct, vcscale, frame),
            CircuitElement::Device(d) => d.draw_persistent(vct, vcscale, frame),
        }
    }

    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            CircuitElement::NetEdge(e) => e.draw_selected(vct, vcscale, frame),
            CircuitElement::Device(d) => d.draw_selected(vct, vcscale, frame),
        }
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            CircuitElement::NetEdge(e) => e.draw_preview(vct, vcscale, frame),
            CircuitElement::Device(d) => d.draw_preview(vct, vcscale, frame),
        }
    }
}

impl SchematicElement for CircuitElement {
    fn contains_ssp(&self, ssp: SSPoint) -> bool {
        match self {
            CircuitElement::NetEdge(e) => e.interactable.contains_ssp(ssp),
            CircuitElement::Device(d) => d.0.borrow().interactable.contains_ssp(ssp),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    CanvasEvent(Event, SSPoint),
    Wire,
    NetList,
    DcOp(PkVecvaluesall),
}

impl schematic::ContentMsg for Msg {
    fn canvas_event_msg(event: Event, curpos_ssp: SSPoint) -> Self {
        Msg::CanvasEvent(event, curpos_ssp)
    }
}

#[derive(Debug, Clone, Default)]
pub enum CircuitSt {
    #[default]
    Idle,
    Wiring(Option<(Box<Nets>, SSPoint)>),
}

impl CircuitSt {
    fn move_transform(ssp0: &SSPoint, ssp1: &SSPoint, sst: &SSTransform) -> SSTransform {
        sst.pre_translate(SSVec::new(-ssp0.x, -ssp0.y))
            .then_translate(SSVec::new(ssp0.x, ssp0.y))
            .then_translate(*ssp1 - *ssp0)
    }
}

/// struct holding schematic state (nets, devices, and their locations)
#[derive(Debug, Default, Clone)]
pub struct Circuit {
    pub infobarstr: Option<String>,

    state: CircuitSt,

    nets: Nets,
    devices: Devices,
    curpos_ssp: SSPoint,
}

impl Circuit {
    fn update_cursor_ssp(&mut self, curpos_ssp: SSPoint) {
        self.curpos_ssp = curpos_ssp;
        match &mut self.state {
            CircuitSt::Wiring(Some((nets, ssp_prev))) => {
                nets.clear();
                nets.route(*ssp_prev, curpos_ssp);
            }
            CircuitSt::Idle => {}
            _ => {}
        }
    }
}

impl Drawable for Circuit {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.nets.draw_persistent(vct, vcscale, frame);
        self.devices.draw_persistent(vct, vcscale, frame);
    }

    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        panic!("not intended for use");
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match &self.state {
            CircuitSt::Wiring(Some((nets, _))) => {
                nets.draw_preview(vct, vcscale, frame);
            }
            CircuitSt::Idle => {}
            _ => {}
        }
    }
}

impl schematic::Content<CircuitElement, Msg> for Circuit {
    fn bounds(&self) -> VSBox {
        let bbn = self.nets.bounding_box();
        let bbi = self.devices.bounding_box();
        bbn.union(&bbi)
    }
    fn intersects_ssb(&mut self, ssb: SSBox) -> HashSet<CircuitElement> {
        let mut ret = HashSet::new();
        for seg in self.nets.intersects_ssbox(&ssb) {
            ret.insert(CircuitElement::NetEdge(seg));
        }
        for rcrd in self.devices.intersects_ssb(&ssb) {
            ret.insert(CircuitElement::Device(rcrd));
        }
        ret
    }

    fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.nets.occupies_ssp(ssp) || self.devices.occupies_ssp(ssp)
    }

    /// returns the first CircuitElement after skip which intersects with curpos_ssp, if any.
    /// count is updated to track the number of elements skipped over
    fn selectable(
        &mut self,
        ssp: SSPoint,
        skip: usize,
        count: &mut usize,
    ) -> Option<CircuitElement> {
        if let Some(e) = self.nets.selectable(ssp, skip, count) {
            return Some(CircuitElement::NetEdge(e));
        }
        if let Some(d) = self.devices.selectable(ssp, skip, count) {
            return Some(CircuitElement::Device(d));
        }
        None
    }

    fn update(&mut self, msg: Msg) -> SchematicMsg<CircuitElement> {
        let ret_msg = match msg {
            Msg::CanvasEvent(event, curpos_ssp) => {
                if let Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = event {
                    self.update_cursor_ssp(curpos_ssp);
                }

                let mut state = self.state.clone();
                let mut ret_msg_tmp = SchematicMsg::None;
                match (&mut state, event) {
                    // wiring
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::W,
                            modifiers: _,
                        }),
                    ) => {
                        state = CircuitSt::Wiring(None);
                    }
                    (
                        CircuitSt::Wiring(opt_ws),
                        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
                    ) => {
                        let ssp = curpos_ssp;
                        let new_ws;
                        if let Some((g, prev_ssp)) = opt_ws {
                            // subsequent click
                            if ssp == *prev_ssp {
                                new_ws = None;
                            } else if self.occupies_ssp(ssp) {
                                self.nets.merge(g.as_ref(), self.devices.ports_ssp());
                                new_ws = None;
                            } else {
                                self.nets.merge(g.as_ref(), self.devices.ports_ssp());
                                new_ws = Some((Box::<Nets>::default(), ssp));
                            }
                            ret_msg_tmp = SchematicMsg::ClearPassive;
                        } else {
                            // first click
                            new_ws = Some((Box::<Nets>::default(), ssp));
                        }
                        state = CircuitSt::Wiring(new_ws);
                    }
                    // device placement
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::R,
                            modifiers: _,
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
                            modifiers: _,
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
                            modifiers: _,
                        }),
                    ) => {
                        let d = self.devices.new_vs();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(CircuitElement::Device(d)));
                    }
                    // state reset
                    (
                        _,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Escape,
                            modifiers: _,
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
        };
        ret_msg
    }

    fn move_elements(&mut self, elements: &HashSet<CircuitElement>, sst: &SSTransform) {
        for e in elements {
            match e {
                CircuitElement::NetEdge(e) => {
                    self.nets.transform(e.clone(), *sst);
                }
                CircuitElement::Device(d) => {
                    d.0.borrow_mut().transform(*sst);
                    // if moving an existing device, does nothing
                    // inserts the device if placing a new device
                    self.devices.insert(d.clone());
                }
            }
        }
        self.prune();
    }

    fn copy_elements(&mut self, elements: &HashSet<CircuitElement>, sst: &SSTransform) {
        for e in elements {
            match e {
                CircuitElement::NetEdge(seg) => {
                    let mut seg = seg.clone();
                    seg.transform(*sst);
                    self.nets
                        .graph
                        .add_edge(NetVertex(seg.src), NetVertex(seg.dst), seg.clone());
                }
                CircuitElement::Device(rcr) => {
                    //unwrap refcell
                    let refcell_d = rcr.0.borrow();
                    let mut device = (*refcell_d).clone();
                    device.transform(*sst);

                    //build BaseElement
                    let d_refcell = RefCell::new(device);
                    let d_refcnt = Rc::new(d_refcell);
                    let rcr_device = RcRDevice(d_refcnt);
                    self.devices.insert(rcr_device);
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
                    self.devices.delete_device(d);
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
        for d in self.devices.get_set() {
            netlist.push_str(&d.0.borrow_mut().spice_line(&mut self.nets));
        }
        if netlist == "Netlist Created by Circe\n" {  // empty netlist
            netlist.push_str("V_0 0 n1 0");  // give it something so spice doesnt hang
        }
        netlist.push('\n');
        fs::write("netlist.cir", netlist.as_bytes()).expect("Unable to write file");
    }
    /// clear up nets graph: merging segments, cleaning up segment net names, etc.
    fn prune(&mut self) {
        self.nets.prune(self.devices.ports_ssp());
    }
}
