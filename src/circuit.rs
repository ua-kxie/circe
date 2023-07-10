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

    fn set_tentative(&mut self) {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Event(Event, SSPoint),
    Wire,
    DcOp,
    None,
}

impl schematic::ContentMsg for Msg {
    fn canvas_event_msg(event: Event, curpos_ssp: SSPoint) -> Self {
        Msg::Event(event, curpos_ssp)
    }
}

#[derive(Debug, Clone, Default)]
pub enum CircuitSt {
    #[default]
    Idle,
    Wiring(Option<(Box<Nets>, SSPoint)>),
    AreaSelect(SSBox),
    Moving(Option<(SSPoint, SSPoint, SSTransform)>),
    // first click, second click, transform for rotation/flip ONLY
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
}

impl Drawable for Circuit {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.nets.draw_persistent(vct, vcscale, frame);
    }

    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.nets.draw_selected(vct, vcscale, frame);
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.nets.draw_preview(vct, vcscale, frame);
    }
}

impl schematic::Content<CircuitElement, Msg> for Circuit {
    fn bounds(&self) -> VSBox {
        let bbn = self.nets.bounding_box();
        let bbi = self.devices.bounding_box();
        bbn.union(&bbi)
    }

    fn clear_tentatives(&mut self) {
        self.nets.clear_tentatives();
        self.devices.clear_tentatives();
    }

    fn tentatives_by_ssbox(&mut self, ssb: SSBox) {
        self.nets.tentatives_by_ssbox(&ssb);
        self.devices.tentatives_by_ssbox(&ssb);
    }

    fn tentatives(&self) -> Vec<CircuitElement> {
        let mut v = vec![];
        for e in self.nets.tentatives() {
            v.push(CircuitElement::NetEdge(e));
        }
        for d in self.devices.tentatives() {
            v.push(CircuitElement::Device(d));
        }
        v
    }

    fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.nets.occupies_ssp(ssp) || self.devices.occupies_ssp(ssp)
    }

    fn delete(&mut self, targets: &HashSet<CircuitElement>) {
        for e in targets {
            match e {
                CircuitElement::NetEdge(edge) => self.nets.delete_edge(edge),
                CircuitElement::Device(d) => self.devices.delete_device(d),
            }
        }
    }

    fn transform(&mut self, targets: &HashSet<CircuitElement>) {
        todo!()
    }

    fn selectable(
        &mut self,
        ssp: SSPoint,
        skip: &mut usize,
        count: &mut usize,
    ) -> Option<CircuitElement> {
        loop {
            let mut count = 0; // tracks the number of skipped elements
            if let Some(e) = self.nets.selectable(ssp, skip, &mut count) {
                return Some(CircuitElement::NetEdge(e));
            }
            if let Some(d) = self.devices.selectable(ssp, skip, &mut count) {
                return Some(CircuitElement::Device(d));
            }
            if count == 0 {
                *skip = count;
                return None;
            }
            *skip -= count;
        }
    }

    fn update(&mut self, msg: Msg) -> SchematicMsg<CircuitElement> {
        let mut clear_passive = false;
        let ret_msg = match msg {
            Msg::Event(event, curpos_ssp) => {
                let mut state = self.state.clone();
                let mut ret_msg = SchematicMsg::None;
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
                        let mut new_ws = None;
                        if let Some((g, prev_ssp)) = opt_ws {
                            // subsequent click
                            if ssp == *prev_ssp {
                            } else if self.occupies_ssp(ssp) {
                                self.nets.merge(g.as_ref(), self.devices.ports_ssp());
                                new_ws = None;
                            } else {
                                self.nets.merge(g.as_ref(), self.devices.ports_ssp());
                                new_ws = Some((Box::<Nets>::default(), ssp));
                            }
                        } else {
                            // first click
                            new_ws = Some((Box::<Nets>::default(), ssp));
                        }
                        state = CircuitSt::Wiring(new_ws);
                        clear_passive = true;
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
                        ret_msg = SchematicMsg::NewElement(CircuitElement::Device(d));
                    }
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::G,
                            modifiers: _,
                        }),
                    ) => {
                        let d = self.devices.new_gnd();
                        ret_msg = SchematicMsg::NewElement(CircuitElement::Device(d));
                    }
                    (
                        CircuitSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::V,
                            modifiers: _,
                        }),
                    ) => {
                        let d = self.devices.new_vs();
                        ret_msg = SchematicMsg::NewElement(CircuitElement::Device(d));
                    }
                    _ => {}
                }
                self.state = state;
                ret_msg
            }
            Msg::Wire => {
                SchematicMsg::None
            },
            Msg::DcOp => {
                SchematicMsg::None
            },
            Msg::None => {
                SchematicMsg::None
            },
        };
        ret_msg
    }

    fn move_elements(&mut self, elements: &HashSet<CircuitElement>, sst: &SSTransform) {
        todo!()
    }

    fn copy_elements(&mut self, elements: &HashSet<CircuitElement>, sst: &SSTransform) {
        todo!()
    }

    fn delete_elements(&mut self, elements: &HashSet<CircuitElement>) {
        todo!()
    }

    fn tentative_by_ssp(&mut self, curpos_ssp: SSPoint) {
        // todo!()
    }
}

impl Circuit {
    /// process dc operating point simulation results - draws the voltage of connected nets near the connected port
    pub fn op(&mut self, pkvecvaluesall: &paprika::PkVecvaluesall) {
        self.devices.op(pkvecvaluesall);
    }
    /// create netlist for the current schematic and save it.
    pub fn netlist(&mut self) {
        self.nets.pre_netlist();
        let mut netlist = String::from("Netlist Created by Circe\n");
        for d in self.devices.get_set() {
            netlist.push_str(&d.0.borrow_mut().spice_line(&mut self.nets));
        }
        netlist.push('\n');
        fs::write("netlist.cir", netlist.as_bytes()).expect("Unable to write file");
    }
    /// clear up nets graph: merging segments, cleaning up segment net names, etc.
    fn prune_nets(&mut self) {
        self.nets.prune(self.devices.ports_ssp());
    }
}
