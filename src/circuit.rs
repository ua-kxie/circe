//! Circuit
//! Concrete types for schematic content

use crate::schematic::nets::{NetEdge, NetVertex, Nets};
use crate::schematic::{self, RcRDevice, SchematicElement};
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
}

#[derive(Debug, Clone, Copy)]
enum DeviceType {
    Gnd,
    V,
    R,
}

#[derive(Debug, Clone, Copy)]
pub enum CircuitMsg {
    Event(Event),
    Wire,
    DcOp,
    NewDevice(DeviceType),
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
    // devices: Devices,
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

impl schematic::Content for Circuit {
    fn bounds(&self) -> VSBox {
        todo!()
    }

    fn update_cursor_ssp(&mut self, ssp: SSPoint) {
        todo!()
    }
}

// impl viewport::ViewportContent<CircuitMsg> for SchematicContent {
//     fn mouse_interaction(&self) -> mouse::Interaction {
//         match self.state {
//             CircuitSt::Idle => mouse::Interaction::default(),
//             CircuitSt::Wiring(_) => mouse::Interaction::Crosshair,
//             CircuitSt::Moving(_) => mouse::Interaction::Grabbing,
//             CircuitSt::AreaSelect(_) => mouse::Interaction::Crosshair,
//         }
//     }

//     fn events_handler(&self, event: Event) -> Option<CircuitMsg> {
//         match (&self.state, event) {
//             (
//                 CircuitSt::Idle,
//                 Event::Keyboard(iced::keyboard::Event::KeyPressed {
//                     key_code: iced::keyboard::KeyCode::W,
//                     modifiers: _,
//                 }),
//             ) => Some(CircuitMsg::Wire),
//             (
//                 CircuitSt::Idle,
//                 Event::Keyboard(iced::keyboard::Event::KeyPressed {
//                     key_code: iced::keyboard::KeyCode::Space,
//                     modifiers: _,
//                 }),
//             ) => Some(CircuitMsg::DcOp),
//             (_, _) => Some(CircuitMsg::Event(event)),
//         }
//     }

//     /// draw onto active cache
//     fn draw_active(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
//         // draw elements which may need to be redrawn at any event
//         self.nets.draw_preview(vct, vcscale, frame); // this draws tentatives - refactor
//         self.devices.draw_preview(vct, vcscale, frame);

//         match &self.state {
//             CircuitSt::Wiring(Some((net, ..))) => {
//                 net.as_ref().draw_preview(vct, vcscale, frame);
//             }
//             CircuitSt::Idle => {}
//             CircuitSt::AreaSelect(ssb) => {
//                 let color = if ssb.height() > 0 {
//                     Color::from_rgba(1., 1., 0., 0.1)
//                 } else {
//                     Color::from_rgba(0., 1., 1., 0.1)
//                 };
//                 let f = canvas::Fill {
//                     style: canvas::Style::Solid(color),
//                     ..canvas::Fill::default()
//                 };
//                 let csb = vct.outer_transformed_box(&ssb.cast().cast_unit());
//                 let size = Size::new(csb.width(), csb.height());
//                 frame.fill_rectangle(Point::from(csb.min).into(), size, f);

//                 let mut path_builder = Builder::new();
//                 path_builder.line_to(Point::from(csb.min).into());
//                 path_builder.line_to(Point::from(CSPoint::new(csb.min.x, csb.max.y)).into());
//                 path_builder.line_to(Point::from(csb.max).into());
//                 path_builder.line_to(Point::from(CSPoint::new(csb.max.x, csb.min.y)).into());
//                 path_builder.line_to(Point::from(csb.min).into());
//                 let stroke = Stroke {
//                     width: (0.1 * vcscale).max(0.1 * 2.0),
//                     style: canvas::stroke::Style::Solid(color),
//                     line_cap: LineCap::Square,
//                     ..Stroke::default()
//                 };
//                 frame.stroke(&path_builder.build(), stroke);
//             }
//             CircuitSt::Moving(Some((ssp0, ssp1, sst))) => {
//                 let vvt = transforms::sst_to_xxt::<ViewportSpace>(
//                     CircuitSt::move_transform(ssp0, ssp1, sst),
//                 );

//                 let vct_c = vvt.then(&vct);
//                 for be in &self.selected {
//                     match be {
//                         CircuitElement::Device(d) => d.0.borrow().draw_preview(vct_c, vcscale, frame),
//                         CircuitElement::NetEdge(e) => e.draw_preview(vct_c, vcscale, frame),
//                     }
//                 }
//             }
//             _ => {}
//         }
//     }
//     /// draw onto passive cache
//     fn draw_passive(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
//         // draw elements which may need to be redrawn at any event
//         self.nets.draw_persistent(vct, vcscale, frame);
//         self.devices.draw_persistent(vct, vcscale, frame);
//         let _: Vec<_> = self
//             .selected
//             .iter()
//             .map(|e| match e {
//                 CircuitElement::NetEdge(e) => {
//                     e.draw_selected(vct, vcscale, frame);
//                 }
//                 CircuitElement::Device(d) => {
//                     d.0.borrow().draw_selected(vct, vcscale, frame);
//                 }
//             })
//             .collect();
//     }

//     /// returns the bouding box of all elements on canvas
//     fn bounds(&self) -> VSBox {
//         let bbn = self.nets.bounding_box();
//         let bbi = self.devices.bounding_box();
//         bbn.union(&bbi)
//     }
//     /// mutate state based on message and cursor position
//     fn update(&mut self, msg: CircuitMsg, curpos_ssp: SSPoint) -> bool {
//         let mut clear_passive = false;
//         match msg {
//             CircuitMsg::Wire => {
//                 self.state = CircuitSt::Wiring(None);
//             }
//             CircuitMsg::Event(event) => {
//                 if let Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = event {
//                     self.update_cursor_ssp(curpos_ssp);
//                 }

//                 let mut state = self.state.clone();
//                 match (&mut state, event) {
//                     // wiring
//                     (
//                         CircuitSt::Wiring(opt_ws),
//                         Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
//                     ) => {
//                         let ssp = curpos_ssp;
//                         let mut new_ws = None;
//                         if let Some((g, prev_ssp)) = opt_ws {
//                             // subsequent click
//                             if ssp == *prev_ssp {
//                             } else if self.occupies_ssp(ssp) {
//                                 self.nets.merge(g.as_ref(), self.devices.ports_ssp());
//                                 new_ws = None;
//                             } else {
//                                 self.nets.merge(g.as_ref(), self.devices.ports_ssp());
//                                 new_ws = Some((Box::<Nets>::default(), ssp));
//                             }
//                         } else {
//                             // first click
//                             new_ws = Some((Box::<Nets>::default(), ssp));
//                         }
//                         state = CircuitSt::Wiring(new_ws);
//                         clear_passive = true;
//                     }

//                     // drag/area select - todo move to viewport - content should allow viewport to discern areaselect or drag
//                     (
//                         CircuitSt::Idle,
//                         Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
//                     ) => {
//                         let mut click_selected = false;

//                         for s in &self.selected {
//                             if let CircuitElement::Device(rcr) = s {
//                                 if rcr.0.borrow().interactable.contains_ssp(curpos_ssp) {
//                                     click_selected = true;
//                                     break;
//                                 }
//                             }
//                         }

//                         if click_selected {
//                             state = CircuitSt::Moving(Some((
//                                 curpos_ssp,
//                                 curpos_ssp,
//                                 SSTransform::identity(),
//                             )));
//                         } else {
//                             state =
//                                 CircuitSt::AreaSelect(SSBox::new(curpos_ssp, curpos_ssp));
//                         }
//                     }

//                     // area select
//                     (
//                         CircuitSt::AreaSelect(_),
//                         Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
//                     ) => {
//                         self.tentatives_to_selected();
//                         state = CircuitSt::Idle;
//                         clear_passive = true;
//                     }
//                     // device placement
//                     (
//                         CircuitSt::Idle,
//                         Event::Keyboard(iced::keyboard::Event::KeyPressed {
//                             key_code: iced::keyboard::KeyCode::R,
//                             modifiers: _,
//                         }),
//                     ) => {
//                         self.selected.clear();
//                         let d = self.devices.new_res();
//                         d.0.borrow_mut().set_position(curpos_ssp);
//                         self.selected.insert(CircuitElement::Device(d));
//                         state = CircuitSt::Moving(Some((
//                             curpos_ssp,
//                             curpos_ssp,
//                             SSTransform::identity(),
//                         )));
//                     }
//                     (
//                         CircuitSt::Idle,
//                         Event::Keyboard(iced::keyboard::Event::KeyPressed {
//                             key_code: iced::keyboard::KeyCode::G,
//                             modifiers: _,
//                         }),
//                     ) => {
//                         self.selected.clear();
//                         let d = self.devices.new_gnd();
//                         d.0.borrow_mut().set_position(curpos_ssp);
//                         self.selected.insert(CircuitElement::Device(d));
//                         state = CircuitSt::Moving(Some((
//                             curpos_ssp,
//                             curpos_ssp,
//                             SSTransform::identity(),
//                         )));
//                     }
//                     (
//                         CircuitSt::Idle,
//                         Event::Keyboard(iced::keyboard::Event::KeyPressed {
//                             key_code: iced::keyboard::KeyCode::V,
//                             modifiers: _,
//                         }),
//                     ) => {
//                         self.selected.clear();
//                         let d = self.devices.new_vs();
//                         d.0.borrow_mut().set_position(curpos_ssp);
//                         self.selected.insert(CircuitElement::Device(d));
//                         state = CircuitSt::Moving(Some((
//                             curpos_ssp,
//                             curpos_ssp,
//                             SSTransform::identity(),
//                         )));
//                     }
//                     // moving
//                     (
//                         _,
//                         Event::Keyboard(iced::keyboard::Event::KeyPressed {
//                             key_code: iced::keyboard::KeyCode::M,
//                             modifiers: _,
//                         }),
//                     ) => {
//                         state = CircuitSt::Moving(None);
//                     }
//                     (
//                         CircuitSt::Moving(Some((_ssp0, _ssp1, sst))),
//                         Event::Keyboard(iced::keyboard::Event::KeyPressed {
//                             key_code: iced::keyboard::KeyCode::R,
//                             modifiers: _,
//                         }),
//                     ) => {
//                         *sst = sst.then(&transforms::SST_CWR);
//                     }
//                     (
//                         CircuitSt::Moving(mut opt_pts),
//                         Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
//                     ) => {
//                         if let Some((ssp0, ssp1, vvt)) = &mut opt_pts {
//                             self.move_selected(CircuitSt::move_transform(ssp0, ssp1, vvt));
//                             self.prune_nets();
//                             state = CircuitSt::Idle;
//                             clear_passive = true;
//                         } else {
//                             let ssp: euclid::Point2D<_, _> = curpos_ssp;
//                             let sst = SSTransform::identity();
//                             state = CircuitSt::Moving(Some((ssp, ssp, sst)));
//                         }
//                     }
//                     // delete - todo move to viewport - send a message down
//                     (
//                         CircuitSt::Idle,
//                         Event::Keyboard(iced::keyboard::Event::KeyPressed {
//                             key_code: iced::keyboard::KeyCode::Delete,
//                             modifiers: _,
//                         }),
//                     ) => {
//                         self.delete_selected();
//                         clear_passive = true;
//                     }
//                     _ => {}
//                 }
//                 self.state = state;
//             }
//             CircuitMsg::DcOp => {
//                 clear_passive = true;
//             }
//         }
//         clear_passive
//     }
//     fn rst(&mut self) -> bool {
//         match self.state {
//             CircuitSt::Idle => {
//                 self.selected.clear();
//                 true
//             }
//             _ => {
//                 self.state = CircuitSt::Idle;
//                 false
//             }
//         }
//     }
//     fn cycle(&mut self, curpos_ssp: SSPoint) -> bool {
//         if let CircuitSt::Idle = self.state {
//             self.infobarstr = self.tentative_next_by_ssp(curpos_ssp);
//             true
//         } else {
//             false
//         }
//     }
// }

// impl SchematicContent {
//     /// process dc operating point simulation results - draws the voltage of connected nets near the connected port
//     pub fn op(&mut self, pkvecvaluesall: &paprika::PkVecvaluesall) {
//         self.devices.op(pkvecvaluesall);
//     }
//     /// update schematic cursor position
//     fn update_cursor_ssp(&mut self, curpos_ssp: SSPoint) {
//         let mut skip = self.selskip;
//         self.infobarstr = self.tentative_by_sspoint(curpos_ssp, &mut skip);

//         let mut stcp = self.state.clone();
//         match &mut stcp {
//             CircuitSt::Wiring(Some((g, prev_ssp))) => {
//                 g.as_mut().clear();
//                 g.route(*prev_ssp, curpos_ssp);
//             }
//             CircuitSt::AreaSelect(ssb) => {
//                 ssb.max = curpos_ssp;
//                 self.tentatives_by_ssbox(ssb);
//             }
//             CircuitSt::Moving(Some((_ssp0, ssp1, _sst))) => {
//                 *ssp1 = curpos_ssp;
//             }
//             _ => {}
//         }
//         self.state = stcp;
//     }
//     /// returns `Some<RcRDevice>` if there is exactly 1 device in selected, otherwise returns none
//     pub fn active_device(&self) -> Option<RcRDevice> {
//         let mut v: Vec<_> = self
//             .selected
//             .iter()
//             .filter_map(|x| match x {
//                 CircuitElement::Device(d) => Some(d.clone()),
//                 _ => None,
//             })
//             .collect();
//         if v.len() == 1 {
//             v.pop()
//         } else {
//             None
//         }
//     }
//     /// clear tentative selections (cursor hover highlight)
//     fn clear_tentatives(&mut self) {
//         self.devices.clear_tentatives();
//         self.nets.clear_tentatives();
//     }
//     /// set tentative flags by intersection with ssb
//     pub fn tentatives_by_ssbox(&mut self, ssb: &SSBox) {
//         self.clear_tentatives();
//         let ssb_p = SSBox::from_points([ssb.min, ssb.max]).inflate(1, 1);
//         self.devices.tentatives_by_ssbox(&ssb_p);
//         self.nets.tentatives_by_ssbox(&ssb_p);
//     }
//     /// set 1 tentative flag by ssp, skipping skip elements which contains ssp. Returns netname if tentative is a net segment
//     pub fn tentative_by_sspoint(&mut self, ssp: SSPoint, skip: &mut usize) -> Option<String> {
//         self.clear_tentatives();
//         if let Some(be) = self.selectable(ssp, skip) {
//             match be {
//                 CircuitElement::NetEdge(e) => {
//                     let mut netedge = e.clone();
//                     let netname = e.label.map(|x| x.as_ref().clone());
//                     netedge.interactable.tentative = true;
//                     self.nets
//                         .graph
//                         .add_edge(NetVertex(e.src), NetVertex(e.dst), netedge);
//                     netname
//                 }
//                 CircuitElement::Device(d) => {
//                     d.0.borrow_mut().interactable.tentative = true;
//                     None
//                 }
//             }
//         } else {
//             None
//         }
//     }
//     /// set 1 tentative flag by ssp, sets flag on next qualifying element. Returns netname i tentative is a net segment
//     pub fn tentative_next_by_ssp(&mut self, ssp: SSPoint) -> Option<String> {
//         let mut skip = self.selskip.wrapping_add(1);
//         let s = self.tentative_by_sspoint(ssp, &mut skip);
//         self.selskip = skip;
//         s
//     }
//     /// put every element with tentative flag set into selected vector
//     fn tentatives_to_selected(&mut self) {
//         let _: Vec<_> = self
//             .devices
//             .tentatives()
//             .map(|d| {
//                 self.selected.insert(CircuitElement::Device(d));
//             })
//             .collect();
//         let _: Vec<_> = self
//             .nets
//             .tentatives()
//             .map(|e| {
//                 self.selected.insert(CircuitElement::NetEdge(e));
//             })
//             .collect();
//     }
//     /// returns true if ssp is occupied by an element
//     fn occupies_ssp(&self, ssp: SSPoint) -> bool {
//         self.nets.occupies_ssp(ssp) || self.devices.occupies_ssp(ssp)
//     }
//     /// set 1 tentative flag based on ssp and skip number. Returns the flagged element, if any.
//     fn selectable(&mut self, ssp: SSPoint, skip: &mut usize) -> Option<CircuitElement> {
//         loop {
//             let mut count = 0; // tracks the number of skipped elements
//             if let Some(e) = self.nets.selectable(ssp, skip, &mut count) {
//                 return Some(e);
//             }
//             if let Some(d) = self.devices.selectable(ssp, skip, &mut count) {
//                 return Some(d);
//             }
//             if count == 0 {
//                 *skip = count;
//                 return None;
//             }
//             *skip -= count;
//         }
//     }
//     /// delete all elements which appear in the selected array
//     pub fn delete_selected(&mut self) {
//         if let CircuitSt::Idle = self.state {
//             for be in &self.selected {
//                 match be {
//                     CircuitElement::NetEdge(e) => {
//                         self.nets.delete_edge(e);
//                     }
//                     CircuitElement::Device(d) => {
//                         self.devices.delete_device(d);
//                     }
//                 }
//             }
//             self.selected.clear();
//             self.prune_nets();
//         }
//     }
//     /// create netlist for the current schematic and save it.
//     pub fn netlist(&mut self) {
//         self.nets.pre_netlist();
//         let mut netlist = String::from("Netlist Created by Circe\n");
//         for d in self.devices.get_set() {
//             netlist.push_str(&d.0.borrow_mut().spice_line(&mut self.nets));
//         }
//         netlist.push('\n');
//         fs::write("netlist.cir", netlist.as_bytes()).expect("Unable to write file");
//     }
//     /// clear up nets graph: merging segments, cleaning up segment net names, etc.
//     fn prune_nets(&mut self) {
//         self.nets.prune(self.devices.ports_ssp());
//     }
//     /// move all elements in the selected array by sst
//     fn move_selected(&mut self, sst: SSTransform) {
//         let selected = self.selected.clone();
//         self.selected.clear();
//         for be in selected {
//             match be {
//                 CircuitElement::NetEdge(e) => {
//                     self.nets.transform(e, sst); // how to handle copying? e.g. adds new nets
//                 }
//                 CircuitElement::Device(d) => {
//                     d.0.borrow_mut().transform(sst);
//                     // if moving an existing device, does nothing
//                     // inserts the device if placing a new device
//                     self.devices.insert(d);
//                 }
//             }
//         }
//     }
// }
