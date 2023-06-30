//! Schematic
//! Space in which devices and nets live in

mod devices;
mod nets;

use self::devices::Devices;
pub use self::devices::RcRDevice;
use crate::{interactable, viewport, IcedStruct};
use crate::{
    interactable::Interactive,
    transforms::{
        self, CSBox, CSPoint, Point, SSBox, SSPoint, SSTransform, SSVec, VCTransform, VSBox,
        ViewportSpace,
    },
    viewport::{Drawable, Viewport, ViewportState},
};
use iced::widget::row;
use iced::Length;
use iced::{
    mouse,
    widget::canvas::{
        self,
        event::{self, Event},
        path::Builder,
        Cache, Cursor, Frame, Geometry, LineCap, Stroke,
    },
    Color, Rectangle, Size, Theme,
};
use nets::{NetEdge, NetVertex, Nets};
use std::sync::Arc;
use std::{collections::HashSet, fs, process};

use colored::Colorize;
use paprika::*;

/// Spice Manager to facillitate interaction with NgSpice
struct SpManager {
    tmp: Option<PkVecvaluesall>,
}

impl SpManager {
    fn new() -> Self {
        SpManager { tmp: None }
    }
}

#[allow(unused_variables)]
impl paprika::PkSpiceManager for SpManager {
    fn cb_send_char(&mut self, msg: String, id: i32) {
        let opt = msg.split_once(' ');
        let (token, msgs) = match opt {
            Some(tup) => (tup.0, tup.1),
            None => (msg.as_str(), msg.as_str()),
        };
        let msgc = match token {
            "stdout" => msgs.green(),
            "stderr" => msgs.red(),
            _ => msg.magenta().strikethrough(),
        };
        println!("{}", msgc);
    }
    fn cb_send_stat(&mut self, msg: String, id: i32) {
        println!("{}", msg.blue());
    }
    fn cb_ctrldexit(&mut self, status: i32, is_immediate: bool, is_quit: bool, id: i32) {}
    fn cb_send_init(&mut self, pkvecinfoall: PkVecinfoall, id: i32) {}
    fn cb_send_data(&mut self, pkvecvaluesall: PkVecvaluesall, count: i32, id: i32) {
        self.tmp = Some(pkvecvaluesall);
    }
    fn cb_bgt_state(&mut self, is_fin: bool, id: i32) {}
}

/// trait for a type of element in schematic. e.g. nets or devices
pub trait SchematicSet {
    /// returns the first BaseElement after skip which intersects with curpos_ssp, if any.
    fn selectable(
        &mut self,
        curpos_ssp: SSPoint,
        skip: &mut usize,
        count: &mut usize,
    ) -> Option<BaseElement>;

    /// returns the bounding box of all contained elements
    fn bounding_box(&self) -> VSBox;
}

#[derive(Debug, Clone)]
pub enum BaseElement {
    NetEdge(NetEdge),
    Device(RcRDevice),
}

impl PartialEq for BaseElement {
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

impl Eq for BaseElement {}

impl std::hash::Hash for BaseElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            BaseElement::NetEdge(e) => e.hash(state),
            BaseElement::Device(d) => by_address::ByAddress(d).hash(state),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SchematicMsg {
    TextInputChanged(String),
    TextInputSubmit,
    Fit(CSBox),
    ViewportMsg(viewport::ViewportMsg),
    SchematicEvt(Event, SSPoint),
}

#[derive(Clone)]
pub enum SchematicState {
    Wiring(Option<(Box<Nets>, SSPoint)>),
    Idle,
    AreaSelect(SSBox),
    Moving(Option<(SSPoint, SSPoint, SSTransform)>),
    // first click, second click, transform for rotation/flip ONLY
}

impl Default for SchematicState {
    fn default() -> Self {
        SchematicState::Idle
    }
}

impl SchematicState {
    fn move_transform(ssp0: &SSPoint, ssp1: &SSPoint, sst: &SSTransform) -> SSTransform {
        sst.pre_translate(SSVec::new(-ssp0.x, -ssp0.y))
            .then_translate(SSVec::new(ssp0.x, ssp0.y))
            .then_translate(*ssp1 - *ssp0)
    }
}

/// schematic
pub struct Schematic {
    /// iced canvas graphical cache, cleared every frame
    active_cache: Cache,
    /// iced canvas graphical cache, cleared following some schematic actions
    passive_cache: Cache,
    /// iced canvas graphical cache, almost never cleared
    background_cache: Cache,

    /// viewport
    viewport: Viewport,

    /// schematic cursor position
    curpos_ssp: SSPoint,
    /// tentative net name, used only for display in the infobar
    net_name: Option<String>,
    /// active device - some if only 1 device selected, otherwise is none
    active_device: Option<RcRDevice>,
    /// parameter editor text
    text: String,

    /// spice manager
    spmanager: Arc<SpManager>,
    /// ngspice library
    lib: PkSpice<SpManager>,

    nets: Nets,
    devices: Devices,
    state: SchematicState,

    selskip: usize,
    selected: HashSet<BaseElement>,
}
impl Default for Schematic {
    fn default() -> Self {
        let spmanager = Arc::new(SpManager::new());
        let mut lib;
        #[cfg(target_family = "windows")]
        {
            lib = PkSpice::<SpManager>::new(std::ffi::OsStr::new("paprika/ngspice.dll")).unwrap();
        }
        #[cfg(target_os = "macos")]
        {
            // retrieve libngspice.dylib from the following possible directories
            let ret = process::Command::new("find")
                .args(&["/usr/lib", "/usr/local/lib"])
                .arg("-name")
                .arg("*libngspice.dylib")
                .stdout(process::Stdio::piped())
                .output()
                .unwrap_or_else(|_| {
                    eprintln!("Error: Could not find libngspice.dylib. Make sure it is installed.");
                    process::exit(1);
                });
            let path = String::from_utf8(ret.stdout).unwrap();
            lib = PkSpice::<SpManager>::new(&std::ffi::OsString::from(path.trim())).unwrap();
        }
        #[cfg(target_os = "linux")]
        {
            // dynamically retrieves libngspice from system
            let ret = process::Command::new("sh")
                .arg("-c")
                .arg("ldconfig -p | grep ngspice | awk '/.*libngspice.so$/{print $4}'")
                .stdout(process::Stdio::piped())
                .output()
                .unwrap_or_else(|_| {
                    eprintln!("Error: Could not find libngspice. Make sure it is installed.");
                    process::exit(1);
                });

            let path = String::from_utf8(ret.stdout).unwrap();
            lib = PkSpice::<SpManager>::new(&std::ffi::OsString::from(path.trim())).unwrap();
        }
        lib.init(Some(spmanager.clone()));
        Schematic {
            active_cache: Default::default(),
            passive_cache: Default::default(),
            background_cache: Default::default(),
            viewport: Default::default(),
            curpos_ssp: Default::default(),
            net_name: Default::default(),
            active_device: Default::default(),
            text: Default::default(),
            spmanager,
            lib,
            nets: Default::default(),
            devices: Default::default(),
            state: Default::default(),
            selskip: Default::default(),
            selected: Default::default(),
        }
    }
}

impl canvas::Program<SchematicMsg> for Schematic {
    type State = ViewportState;

    fn update(
        &self,
        viewport_st: &mut ViewportState,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<SchematicMsg>) {
        let curpos = cursor.position_in(&bounds);
        let mut msg = None;
        let csb = CSBox::from_points([CSPoint::origin(), CSPoint::new(bounds.x, bounds.y)]);
        self.active_cache.clear();

        if let Some(p) = curpos {
            if let Some(msg) =
                self.viewport
                    .events_handler(viewport_st, event, csb, Point::from(p).into())
            {
                return (
                    event::Status::Captured,
                    Some(SchematicMsg::ViewportMsg(msg)),
                );
            }
        }

        // if let Some(curpos_csp) = curpos.map(|x| Point::from(x).into()) {
        if curpos.is_some() {
            let mut state = self.state.clone();
            match (&mut state, event) {
                // drawing line
                (
                    _,
                    Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key_code: iced::keyboard::KeyCode::F,
                        modifiers: _,
                    }),
                ) => {
                    msg = Some(SchematicMsg::Fit(CSBox::from_points([
                        CSPoint::origin(),
                        CSPoint::new(bounds.x, bounds.y),
                    ])));
                }
                _ => {}
            }
            if msg.is_none() {
                msg = Some(SchematicMsg::SchematicEvt(
                    event,
                    self.viewport.curpos_ssp(),
                ));
            }
        }

        if msg.is_some() {
            (event::Status::Captured, msg)
        } else {
            (event::Status::Ignored, msg)
        }
    }

    fn draw(
        &self,
        viewport_st: &ViewportState,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let active = self.active_cache.draw(bounds.size(), |frame| {
            self.draw_active(
                self.viewport.vc_transform(),
                self.viewport.vc_scale(),
                frame,
            );
            self.viewport.draw_cursor(frame);

            if let ViewportState::NewView(vsp0, vsp1) = viewport_st {
                let csp0 = self.viewport.vc_transform().transform_point(*vsp0);
                let csp1 = self.viewport.vc_transform().transform_point(*vsp1);
                let selsize = Size {
                    width: csp1.x - csp0.x,
                    height: csp1.y - csp0.y,
                };
                let f = canvas::Fill {
                    style: canvas::Style::Solid(if selsize.height > 0. {
                        Color::from_rgba(1., 0., 0., 0.1)
                    } else {
                        Color::from_rgba(0., 0., 1., 0.1)
                    }),
                    ..canvas::Fill::default()
                };
                frame.fill_rectangle(Point::from(csp0).into(), selsize, f);
            }
        });

        let passive = self.passive_cache.draw(bounds.size(), |frame| {
            self.viewport.draw_grid(
                frame,
                CSBox::new(
                    CSPoint::origin(),
                    CSPoint::from([bounds.width, bounds.height]),
                ),
            );
            self.draw_passive(
                self.viewport.vc_transform(),
                self.viewport.vc_scale(),
                frame,
            );
        });

        let background = self.background_cache.draw(bounds.size(), |frame| {
            let f = canvas::Fill {
                style: canvas::Style::Solid(Color::from_rgb(0.2, 0.2, 0.2)),
                ..canvas::Fill::default()
            };
            frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), f);
        });

        vec![background, passive, active]
    }

    fn mouse_interaction(
        &self,
        viewport_st: &ViewportState,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(&bounds) {
            match (&viewport_st, &self.state) {
                (ViewportState::Panning(_), _) => mouse::Interaction::Grabbing,
                (ViewportState::None, SchematicState::Idle) => mouse::Interaction::default(),
                (ViewportState::None, SchematicState::Wiring(_)) => mouse::Interaction::Crosshair,
                (ViewportState::None, SchematicState::Moving(_)) => {
                    mouse::Interaction::ResizingVertically
                }
                _ => mouse::Interaction::default(),
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl IcedStruct<SchematicMsg> for Schematic {
    fn update(&mut self, msg: SchematicMsg) {
        match msg {
            SchematicMsg::TextInputChanged(s) => {
                self.text = s;
            }
            SchematicMsg::TextInputSubmit => {
                if let Some(ad) = &self.active_device {
                    ad.0.borrow_mut().class_mut().set(self.text.clone());
                    self.passive_cache.clear();
                }
            }
            SchematicMsg::Fit(csb) => {
                let vsb = self.bounding_box().inflate(5.0, 5.0);
                let csp = self.viewport.curpos_csp();
                self.viewport
                    .update(self.viewport.display_bounds(csb, vsb, csp));
                self.passive_cache.clear();
            }
            SchematicMsg::ViewportMsg(viewport_msg) => {
                self.viewport.update(viewport_msg);
                self.update_cursor_ssp(self.viewport.curpos_ssp());
                self.passive_cache.clear();
            }
            SchematicMsg::SchematicEvt(event, curpos_ssp) => {
                self.events_handler(event, curpos_ssp);

                self.active_device = self.active_device();
                if let Some(rcrd) = &self.active_device {
                    self.text = rcrd.0.borrow().class().param_summary();
                } else {
                    self.text = String::from("");
                }
            }
        }
    }

    fn view(&self) -> iced::Element<SchematicMsg> {
        let str_ssp = format!("x: {}; y: {}", self.curpos_ssp.x, self.curpos_ssp.y);
        let net_name = self.net_name.as_deref().unwrap_or_default();

        let canvas = iced::widget::canvas(self)
            .width(Length::Fill)
            .height(Length::Fill);
        let pe =
            param_editor::param_editor(self.text.clone(), SchematicMsg::TextInputChanged, || {
                SchematicMsg::TextInputSubmit
            });
        // let mut pe = text("");
        // if let Some(active_device) = &self.active_device {
        //     active_device.0.borrow().class().
        // }
        let schematic = iced::widget::row![
            pe,
            iced::widget::column![
                canvas,
                row![
                    iced::widget::text(str_ssp)
                        .size(16)
                        .height(16)
                        .vertical_alignment(iced::alignment::Vertical::Center),
                    iced::widget::text(&format!("{:04.1}", self.viewport.vc_scale()))
                        .size(16)
                        .height(16)
                        .vertical_alignment(iced::alignment::Vertical::Center),
                    iced::widget::text(net_name)
                        .size(16)
                        .height(16)
                        .vertical_alignment(iced::alignment::Vertical::Center),
                ]
                .spacing(10)
            ]
            .width(Length::Fill)
        ];
        schematic.into()
    }
}

impl Schematic {
    /// update schematic cursor position
    fn update_cursor_ssp(&mut self, ssp: SSPoint) {
        let mut skip = self.selskip.saturating_sub(1);
        self.net_name = self.tentative_by_sspoint(ssp, &mut skip);
        self.selskip = skip;

        let mut st = self.state.clone();
        match &mut st {
            SchematicState::Wiring(Some((g, prev_ssp))) => {
                g.as_mut().clear();
                g.route(*prev_ssp, ssp);
            }
            SchematicState::AreaSelect(ssb) => {
                ssb.max = ssp;
                self.tentatives_by_ssbox(ssb);
            }
            SchematicState::Moving(Some((_ssp0, ssp1, _sst))) => {
                *ssp1 = ssp;
            }
            _ => {}
        }
        self.state = st;
    }
    /// returns `Some<RcRDevice>` if there is exactly 1 device in selected, otherwise returns none
    pub fn active_device(&self) -> Option<RcRDevice> {
        let mut v: Vec<_> = self
            .selected
            .iter()
            .filter_map(|x| match x {
                BaseElement::Device(d) => Some(d.clone()),
                _ => None,
            })
            .collect();
        if v.len() == 1 {
            v.pop()
        } else {
            None
        }
    }
    /// clear selection
    fn clear_selected(&mut self) {
        self.selected.clear();
    }
    /// clear tentative selections (cursor hover highlight)
    fn clear_tentatives(&mut self) {
        self.devices.clear_tentatives();
        self.nets.clear_tentatives();
    }
    /// set tentative flags by intersection with ssb
    pub fn tentatives_by_ssbox(&mut self, ssb: &SSBox) {
        self.clear_tentatives();
        let ssb_p = SSBox::from_points([ssb.min, ssb.max]).inflate(1, 1);
        self.devices.tentatives_by_ssbox(&ssb_p);
        self.nets.tentatives_by_ssbox(&ssb_p);
    }
    /// set 1 tentative flag by ssp, skipping skip elements which contains ssp. Returns netname if tentative is a net segment
    pub fn tentative_by_sspoint(&mut self, ssp: SSPoint, skip: &mut usize) -> Option<String> {
        self.clear_tentatives();
        if let Some(be) = self.selectable(ssp, skip) {
            match be {
                BaseElement::NetEdge(e) => {
                    let mut netedge = e.clone();
                    let netname = e.label.map(|x| x.as_ref().clone());
                    netedge.interactable.tentative = true;
                    self.nets
                        .graph
                        .add_edge(NetVertex(e.src), NetVertex(e.dst), netedge);
                    netname
                }
                BaseElement::Device(d) => {
                    d.0.borrow_mut().interactable.tentative = true;
                    None
                }
            }
        } else {
            None
        }
    }
    /// set 1 tentative flag by ssp, sets flag on next qualifying element. Returns netname i tentative is a net segment
    pub fn tentative_next_by_ssp(&mut self, ssp: SSPoint) -> Option<String> {
        let mut skip = self.selskip;
        let s = self.tentative_by_sspoint(ssp, &mut skip);
        self.selskip = skip;
        s
    }
    /// put every element with tentative flag set into selected vector
    fn tentatives_to_selected(&mut self) {
        let _: Vec<_> = self
            .devices
            .tentatives()
            .map(|d| {
                self.selected.insert(BaseElement::Device(d));
            })
            .collect();
        let _: Vec<_> = self
            .nets
            .tentatives()
            .map(|e| {
                self.selected.insert(BaseElement::NetEdge(e));
            })
            .collect();
    }
    /// returns true if ssp is occupied by an element
    fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.nets.occupies_ssp(ssp) || self.devices.occupies_ssp(ssp)
    }
    /// draw onto active cache
    pub fn draw_active(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        // draw elements which may need to be redrawn at any event
        self.nets.draw_preview(vct, vcscale, frame); // this draws tentatives - refactor
        self.devices.draw_preview(vct, vcscale, frame);

        match &self.state {
            SchematicState::Wiring(Some((net, ..))) => {
                net.as_ref().draw_preview(vct, vcscale, frame);
            }
            SchematicState::Idle => {}
            SchematicState::AreaSelect(ssb) => {
                let color = if ssb.height() > 0 {
                    Color::from_rgba(1., 1., 0., 0.1)
                } else {
                    Color::from_rgba(0., 1., 1., 0.1)
                };
                let f = canvas::Fill {
                    style: canvas::Style::Solid(color),
                    ..canvas::Fill::default()
                };
                let csb = vct.outer_transformed_box(&ssb.cast().cast_unit());
                let size = Size::new(csb.width(), csb.height());
                frame.fill_rectangle(Point::from(csb.min).into(), size, f);

                let mut path_builder = Builder::new();
                path_builder.line_to(Point::from(csb.min).into());
                path_builder.line_to(Point::from(CSPoint::new(csb.min.x, csb.max.y)).into());
                path_builder.line_to(Point::from(csb.max).into());
                path_builder.line_to(Point::from(CSPoint::new(csb.max.x, csb.min.y)).into());
                path_builder.line_to(Point::from(csb.min).into());
                let stroke = Stroke {
                    width: (0.1 * vcscale).max(0.1 * 2.0),
                    style: canvas::stroke::Style::Solid(color),
                    line_cap: LineCap::Square,
                    ..Stroke::default()
                };
                frame.stroke(&path_builder.build(), stroke);
            }
            SchematicState::Moving(Some((ssp0, ssp1, sst))) => {
                let vvt = transforms::sst_to_xxt::<ViewportSpace>(SchematicState::move_transform(
                    ssp0, ssp1, sst,
                ));

                let vct_c = vvt.then(&vct);
                for be in &self.selected {
                    match be {
                        BaseElement::Device(d) => d.0.borrow().draw_preview(vct_c, vcscale, frame),
                        BaseElement::NetEdge(e) => e.draw_preview(vct_c, vcscale, frame),
                    }
                }
            }
            _ => {}
        }
    }
    /// draw onto passive cache
    pub fn draw_passive(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        // draw elements which may need to be redrawn at any event
        self.nets.draw_persistent(vct, vcscale, frame);
        self.devices.draw_persistent(vct, vcscale, frame);
        let _: Vec<_> = self
            .selected
            .iter()
            .map(|e| match e {
                BaseElement::NetEdge(e) => {
                    e.draw_selected(vct, vcscale, frame);
                }
                BaseElement::Device(d) => {
                    d.0.borrow().draw_selected(vct, vcscale, frame);
                }
            })
            .collect();
    }
    /// returns the bouding box of all elements on canvas
    pub fn bounding_box(&self) -> VSBox {
        let bbn = self.nets.bounding_box();
        let bbi = self.devices.bounding_box();
        bbn.union(&bbi)
    }
    /// set 1 tentative flag based on ssp and skip number. Returns the flagged element, if any.
    fn selectable(&mut self, ssp: SSPoint, skip: &mut usize) -> Option<BaseElement> {
        loop {
            let mut count = 0;
            if let Some(e) = self.nets.selectable(ssp, skip, &mut count) {
                return Some(e);
            }
            if let Some(d) = self.devices.selectable(ssp, skip, &mut count) {
                return Some(d);
            }
            if count == 0 {
                *skip = count;
                return None;
            }
            *skip -= count;
        }
    }
    /// delete all elements which appear in the selected array
    pub fn delete_selected(&mut self) {
        if let SchematicState::Idle = self.state {
            for be in &self.selected {
                match be {
                    BaseElement::NetEdge(e) => {
                        self.nets.delete_edge(e);
                    }
                    BaseElement::Device(d) => {
                        self.devices.delete_device(d);
                    }
                }
            }
            self.selected.clear();
            self.prune_nets();
        }
    }
    /// create netlist for the current schematic and save it.
    fn netlist(&mut self) {
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
    /// move all elements in the selected array by sst
    fn move_selected(&mut self, sst: SSTransform) {
        let selected = self.selected.clone();
        self.selected.clear();
        for be in selected {
            match be {
                BaseElement::NetEdge(e) => {
                    self.nets.transform(e, sst); // how to handle copying? e.g. adds new nets
                }
                BaseElement::Device(d) => {
                    d.0.borrow_mut().transform(sst);
                    // if moving an existing device, does nothing
                    // inserts the device if placing a new device
                    self.devices.insert(d);
                }
            }
        }
    }
    /// mutate schematic based on event
    pub fn events_handler(&mut self, event: Event, curpos_ssp: SSPoint) -> (Option<String>, bool) {
        let mut ret = None;
        let mut clear_passive = false;

        if let Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = event {
            let mut skip = self.selskip.saturating_sub(1);
            ret = self.tentative_by_sspoint(curpos_ssp, &mut skip);
            self.selskip = skip;
        }

        let mut state = self.state.clone();
        match (&mut state, event) {
            // wiring
            (
                _,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::W,
                    modifiers: _,
                }),
            ) => {
                state = SchematicState::Wiring(None);
            }

            (
                SchematicState::Wiring(Some((g, prev_ssp))),
                Event::Mouse(iced::mouse::Event::CursorMoved { .. }),
            ) => {
                g.as_mut().clear();
                g.route(*prev_ssp, curpos_ssp);
            }
            (
                SchematicState::Wiring(opt_ws),
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
                state = SchematicState::Wiring(new_ws);
                clear_passive = true;
            }
            // selecting
            (
                SchematicState::Idle,
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
            ) => {
                let mut click_selected = false;

                for s in &self.selected {
                    if let BaseElement::Device(rcr) = s {
                        if rcr.0.borrow().interactable.contains_ssp(curpos_ssp) {
                            click_selected = true;
                            break;
                        }
                    }
                }

                if click_selected {
                    state = SchematicState::Moving(Some((
                        curpos_ssp,
                        curpos_ssp,
                        SSTransform::identity(),
                    )));
                } else {
                    state = SchematicState::AreaSelect(SSBox::new(curpos_ssp, curpos_ssp));
                }
            }
            (
                SchematicState::AreaSelect(ssb),
                Event::Mouse(iced::mouse::Event::CursorMoved { .. }),
            ) => {
                ssb.max = curpos_ssp;
                self.tentatives_by_ssbox(ssb);
            }
            (
                SchematicState::AreaSelect(_),
                Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
            ) => {
                self.tentatives_to_selected();
                state = SchematicState::Idle;
                clear_passive = true;
            }
            // device placement
            (
                SchematicState::Idle,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::R,
                    modifiers: _,
                }),
            ) => {
                self.selected.clear();
                let d = self.devices.new_res();
                d.0.borrow_mut().set_position(curpos_ssp);
                self.selected.insert(BaseElement::Device(d));
                state =
                    SchematicState::Moving(Some((curpos_ssp, curpos_ssp, SSTransform::identity())));
            }
            (
                SchematicState::Idle,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::G,
                    modifiers: _,
                }),
            ) => {
                self.selected.clear();
                let d = self.devices.new_gnd();
                d.0.borrow_mut().set_position(curpos_ssp);
                self.selected.insert(BaseElement::Device(d));
                state =
                    SchematicState::Moving(Some((curpos_ssp, curpos_ssp, SSTransform::identity())));
            }
            (
                SchematicState::Idle,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::V,
                    modifiers: _,
                }),
            ) => {
                self.selected.clear();
                let d = self.devices.new_vs();
                d.0.borrow_mut().set_position(curpos_ssp);
                self.selected.insert(BaseElement::Device(d));
                state =
                    SchematicState::Moving(Some((curpos_ssp, curpos_ssp, SSTransform::identity())));
            }
            // moving
            (
                _,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::M,
                    modifiers: _,
                }),
            ) => {
                state = SchematicState::Moving(None);
            }
            (
                SchematicState::Moving(Some((_ssp0, ssp1, _sst))),
                Event::Mouse(iced::mouse::Event::CursorMoved { .. }),
            ) => {
                *ssp1 = curpos_ssp;
            }
            (
                SchematicState::Moving(Some((_ssp0, _ssp1, sst))),
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::R,
                    modifiers: _,
                }),
            ) => {
                *sst = sst.then(&transforms::SST_CWR);
            }
            (
                SchematicState::Moving(mut opt_pts),
                Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
            ) => {
                if let Some((ssp0, ssp1, vvt)) = &mut opt_pts {
                    self.move_selected(SchematicState::move_transform(ssp0, ssp1, vvt));
                    self.prune_nets();
                    state = SchematicState::Idle;
                    clear_passive = true;
                } else {
                    let ssp: euclid::Point2D<_, _> = curpos_ssp;
                    let sst = SSTransform::identity();
                    state = SchematicState::Moving(Some((ssp, ssp, sst)));
                }
            }
            // esc
            (
                st,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::Escape,
                    modifiers: _,
                }),
            ) => match st {
                SchematicState::Idle => {
                    self.clear_selected();
                    clear_passive = true;
                }
                _ => {
                    state = SchematicState::Idle;
                }
            },
            // delete
            (
                SchematicState::Idle,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::Delete,
                    modifiers: _,
                }),
            ) => {
                self.delete_selected();
                clear_passive = true;
            }
            // cycle
            (
                SchematicState::Idle,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::C,
                    modifiers: _,
                }),
            ) => {
                ret = self.tentative_next_by_ssp(curpos_ssp);
            }
            // test
            (
                SchematicState::Idle,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::T,
                    modifiers: _,
                }),
            ) => {
                self.netlist();
            }
            // dc op
            (
                SchematicState::Idle,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::Space,
                    modifiers: _,
                }),
            ) => {
                self.netlist();
                self.lib.command("source netlist.cir"); // results pointer array starts at same address
                self.lib.command("op"); // ngspice recommends sending in control statements separately, not as part of netlist
                if let Some(pkvecvaluesall) = self.spmanager.tmp.as_ref() {
                    self.devices.op(pkvecvaluesall);
                }
            }
            _ => {}
        }
        self.state = state;
        (ret, clear_passive)
    }
}

mod param_editor {
    use iced::widget::{button, column, text_input};
    use iced::{Element, Length, Renderer};
    use iced_lazy::{component, Component};

    #[derive(Debug, Clone)]
    pub enum Evt {
        InputChanged(String),
        InputSubmit,
    }

    pub struct ParamEditor<Message> {
        value: String,
        on_change: Box<dyn Fn(String) -> Message>,
        on_submit: Box<dyn Fn() -> Message>,
    }

    impl<Message> ParamEditor<Message> {
        pub fn new(
            value: String,
            on_change: impl Fn(String) -> Message + 'static,
            on_submit: impl Fn() -> Message + 'static,
        ) -> Self {
            Self {
                value,
                on_change: Box::new(on_change),
                on_submit: Box::new(on_submit),
            }
        }
    }

    pub fn param_editor<Message>(
        value: String,
        on_change: impl Fn(String) -> Message + 'static,
        on_submit: impl Fn() -> Message + 'static,
    ) -> ParamEditor<Message> {
        ParamEditor::new(value, on_change, on_submit)
    }

    impl<Message> Component<Message, Renderer> for ParamEditor<Message> {
        type State = ();
        type Event = Evt;

        fn update(&mut self, _state: &mut Self::State, event: Evt) -> Option<Message> {
            match event {
                Evt::InputChanged(s) => Some((self.on_change)(s)),
                Evt::InputSubmit => Some((self.on_submit)()),
            }
        }
        fn view(&self, _state: &Self::State) -> Element<Evt, Renderer> {
            column![
                text_input("", &self.value)
                    .width(50)
                    .on_input(Evt::InputChanged)
                    .on_submit(Evt::InputSubmit),
                button("enter"),
            ]
            .width(Length::Shrink)
            .into()
        }
    }

    impl<'a, Message> From<ParamEditor<Message>> for Element<'a, Message, Renderer>
    where
        Message: 'a,
    {
        fn from(parameditor: ParamEditor<Message>) -> Self {
            component(parameditor)
        }
    }
}
