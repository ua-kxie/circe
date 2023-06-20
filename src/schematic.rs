mod nets;
mod devices;
mod interactable;

use std::{collections::HashSet, fs};

use euclid::{Vector2D, Transform2D};
pub use nets::{Drawable, Nets, graph::{NetEdge, NetVertex}};
use crate::transforms::{SSPoint, VCTransform, VSBox, Point, SSBox, SchematicSpace, CSPoint, ViewportSpace, self};
use iced::widget::canvas::{event::Event, path::Builder, Stroke, LineCap};
use iced::{widget::canvas::{
    Frame, self,
}, Size, Color};
pub use self::{devices::{Devices, RcRDevice}, interactable::Interactive};

pub trait SchematicSet {
    fn selectable(&mut self, curpos_ssp: SSPoint, skip: &mut usize, count: &mut usize) -> Option<BaseElement>;
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
            (Self::Device(l0), Self::Device(r0)) => by_address::ByAddress(l0) == by_address::ByAddress(r0),
            _ => false,
        }
    }
}

impl Eq for BaseElement {}

impl std::hash::Hash for BaseElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            BaseElement::NetEdge(e) => {e.hash(state)},
            BaseElement::Device(d) => {by_address::ByAddress(d).hash(state)},
        }
    }
}

#[derive(Clone)]
pub enum SchematicState {
    Wiring(Option<(Box<Nets>, SSPoint)>),
    Idle,
    Selecting(SSBox),
    Moving(Option<(SSPoint, SSPoint, Transform2D<i16, SchematicSpace, SchematicSpace>)>),
    // first click, second click, transform for rotation/flip ONLY
}

impl Default for SchematicState {
    fn default() -> Self {
        SchematicState::Idle
    }
}

impl SchematicState {
    fn move_transform(ssp0: &SSPoint, ssp1: &SSPoint, sst: &Transform2D<i16, SchematicSpace, SchematicSpace>) -> Transform2D<i16, SchematicSpace, SchematicSpace> {
        sst
        .pre_translate(Vector2D::new(-ssp0.x, -ssp0.y))
        .then_translate(Vector2D::new(ssp0.x, ssp0.y))
        .then_translate(*ssp1-*ssp0)
    }
}

#[derive(Default)]
pub struct Schematic {
    nets: Nets,
    devices: Devices,
    pub state: SchematicState,

    selskip: usize,
    selected: HashSet<BaseElement>,
}

impl Schematic {
    pub fn active_device(&self) -> Option<RcRDevice> {
        let mut v: Vec<_> = self.selected.iter().filter_map(|x| {
            match x {
                BaseElement::Device(d) => {Some(d.clone())},
                _ => None,
            }
        }).collect();
        if v.len() == 1 {
            v.pop()
        } else {
            None
        }
    }
    fn clear_selected(&mut self) {
        self.selected.clear();
    }
    fn clear_tentatives(&mut self) {
        self.devices.clear_tentatives();
        self.nets.clear_tentatives();
    }
    pub fn tentatives_by_ssbox(&mut self, ssb: &SSBox) {
        self.clear_tentatives();
        let ssb_p = SSBox::from_points([ssb.min, ssb.max]).inflate(1, 1);
        self.devices.tentatives_by_ssbox(&ssb_p);
        self.nets.tentatives_by_ssbox(&ssb_p);
    }
    pub fn tentative_by_sspoint(&mut self, ssp: SSPoint, skip: &mut usize) -> Option<String> {
        self.clear_tentatives();
        if let Some(be) = self.selectable(ssp, skip) {
            match be {
                BaseElement::NetEdge(e) => {
                    let mut netedge = e.clone();
                    let netname = e.label.map(|x| x.as_ref().clone());
                    netedge.interactable.tentative = true;
                    self.nets.graph.add_edge(NetVertex(e.src), NetVertex(e.dst), netedge);
                    netname
                },
                BaseElement::Device(d) => {
                    d.0.borrow_mut().interactable.tentative = true;
                    None
                },
            }
        } else {None}
    }
    pub fn tentative_next_by_vspoint(&mut self, curpos_ssp: SSPoint) -> Option<String> {
        let mut skip = self.selskip;
        let s = self.tentative_by_sspoint(curpos_ssp, &mut skip);
        self.selskip = skip;
        s
    }
    fn tentatives_to_selected(&mut self) {
        let _: Vec<_> = self.devices.tentatives().map(
            |d| {
                self.selected.insert(BaseElement::Device(d));
            }
        ).collect();
        let _: Vec<_> = self.nets.tentatives().map(
            |e| {
                self.selected.insert(BaseElement::NetEdge(e));
            }
        ).collect();
    }
    fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.nets.occupies_ssp(ssp) || self.devices.occupies_ssp(ssp)
    }
    pub fn draw_active(
        &self, 
        vct: VCTransform,
        vcscale: f32,
        frame: &mut Frame, 
    ) {  // draw elements which may need to be redrawn at any event
        self.nets.draw_preview(vct, vcscale, frame);  // this draws tentatives - refactor
        self.devices.draw_preview(vct, vcscale, frame);

        match &self.state {
            SchematicState::Wiring(Some((net, ..))) => {
                net.as_ref().draw_preview(vct, vcscale, frame);
            },
            SchematicState::Idle => {
            },
            SchematicState::Selecting(ssb) => {
                let color = if ssb.height() > 0 {Color::from_rgba(1., 1., 0., 0.1)} else {Color::from_rgba(0., 1., 1., 0.1)};
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
            },
            SchematicState::Moving(Some((ssp0, ssp1, sst))) => {
                let vvt = SchematicState::move_transform(ssp0, ssp1, sst).cast::<f32>().with_destination::<ViewportSpace>().with_source::<ViewportSpace>();

                let vct_c = vvt.then(&vct);
                for be in &self.selected {
                    match be {
                        BaseElement::Device(d) => {
                            d.0.borrow().draw_preview(vct_c, vcscale, frame)
                        },
                        BaseElement::NetEdge(e) => {
                            e.draw_preview(vct_c, vcscale, frame)
                        }
                    }
                }
            },
            _ => {},
        }
    }

    pub fn draw_passive(
        &self, 
        vct: VCTransform,
        vcscale: f32,
        frame: &mut Frame, 
    ) {  // draw elements which may need to be redrawn at any event
        self.nets.draw_persistent(vct, vcscale, frame);
        self.devices.draw_persistent(vct, vcscale, frame);
        let _: Vec<_> = self.selected.iter().map(|e|
            match e {
                BaseElement::NetEdge(e) => {
                    e.draw_selected(vct, vcscale, frame);
                },
                BaseElement::Device(d) => {
                    d.0.borrow().draw_selected(vct, vcscale, frame);
                },
            }
        ).collect();
    }

    pub fn bounding_box(&self) -> VSBox {
        let bbn = VSBox::from_points(self.nets.graph.nodes().map(|x| x.0.cast().cast_unit()));
        let bbi = self.devices.bounding_box();
        bbn.union(&bbi)
    }

    fn selectable(&mut self, curpos_ssp: SSPoint, skip: &mut usize) -> Option<BaseElement> {
        loop {
            let mut count = 0;
            if let Some(e) = self.nets.selectable(curpos_ssp, skip, &mut count) {
                return Some(e);
            }
            if let Some(d) = self.devices.selectable(curpos_ssp, skip, &mut count) {
                return Some(d);
            }
            if count == 0 {
                *skip = count;
                return None;
            }
            *skip -= count;
        }
    }

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
    pub fn key_test(&mut self) {
        // self.nets.tt();
        let n = self.netlist();
        fs::write("netlist.cir", n.as_bytes()).expect("Unable to write file");
    }
    fn netlist(&mut self) -> String {
        self.nets.pre_netlist();
        let mut netlist = String::from("Netlist Created by Circe\n");
        for d in self.devices.get_set() {
            netlist.push_str(
                &d.0.borrow_mut().spice_line(&mut self.nets)
            );
        }
        netlist.push('\n');
        netlist
    }
    fn prune_nets(&mut self) {
        self.nets.prune(self.devices.ports_ssp());
    }
    fn move_selected(&mut self, sst: Transform2D<i16, SchematicSpace, SchematicSpace>) {
        let selected = self.selected.clone();
        self.selected.clear();
        for be in selected {
            match be {
                BaseElement::NetEdge(e) => {
                    self.nets.transform(e, sst);  // how to handle copying? e.g. adds new nets
                }
                BaseElement::Device(d) => {
                    d.0.borrow_mut().transform(sst);
                    self.devices.insert(d);
                }
            }
        }
    }

    pub fn op(&mut self, pkvecvaluesall: &paprika::PkVecvaluesall) {
        self.devices.op(pkvecvaluesall);
    }

    pub fn events_handler(
        &mut self, 
        event: Event, 
        curpos_ssp: SSPoint, 
    ) -> (Option<String>, bool) {
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
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::W, modifiers: _})
            ) => {
                state = SchematicState::Wiring(None);
            },
            (
                SchematicState::Wiring(Some((g, prev_ssp))), 
                Event::Mouse(iced::mouse::Event::CursorMoved { .. })
            ) => {
                g.as_mut().clear();
                g.route(*prev_ssp, curpos_ssp);
            },
            (
                SchematicState::Wiring(opt_ws), 
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            ) => {
                let ssp = curpos_ssp;
                let mut new_ws = None;
                if let Some((g, prev_ssp)) = opt_ws {  // subsequent click
                    if ssp == *prev_ssp { 
                    } else if self.occupies_ssp(ssp) {
                        self.nets.merge(g.as_ref(), self.devices.ports_ssp());
                        new_ws = None;
                    } else {
                        self.nets.merge(g.as_ref(), self.devices.ports_ssp());
                        new_ws = Some((Box::<Nets>::default(), ssp));
                    }
                } else {  // first click
                    new_ws = Some((Box::<Nets>::default(), ssp));
                }
                state = SchematicState::Wiring(new_ws);
                clear_passive = true;
            },
            // selecting
            (
                SchematicState::Idle, 
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            ) => {
                state = SchematicState::Selecting(SSBox::new(curpos_ssp, curpos_ssp));
            },
            (
                SchematicState::Selecting(ssb), 
                Event::Mouse(iced::mouse::Event::CursorMoved { .. })
            ) => {
                ssb.max = curpos_ssp;
                self.tentatives_by_ssbox(ssb);
            },
            (
                SchematicState::Selecting(_), 
                Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left))
            ) => {
                self.tentatives_to_selected();
                state = SchematicState::Idle;
                clear_passive = true;
            },
            // device placement
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::R, modifiers: _})
            ) => {
                self.selected.clear();
                let d = self.devices.new_res();
                d.0.borrow_mut().set_translation(curpos_ssp);
                self.selected.insert(BaseElement::Device(d));
                state = SchematicState::Moving(Some((curpos_ssp, curpos_ssp, Transform2D::<i16, SchematicSpace, SchematicSpace>::identity())));
            },
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::G, modifiers: _})
            ) => {
                self.selected.clear();
                let d = self.devices.new_gnd();
                d.0.borrow_mut().set_translation(curpos_ssp);
                self.selected.insert(BaseElement::Device(d));
                state = SchematicState::Moving(Some((curpos_ssp, curpos_ssp, Transform2D::<i16, SchematicSpace, SchematicSpace>::identity())));
            },
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::V, modifiers: _})
            ) => {
                self.selected.clear();
                let d = self.devices.new_vs();
                d.0.borrow_mut().set_translation(curpos_ssp);
                self.selected.insert(BaseElement::Device(d));
                state = SchematicState::Moving(Some((curpos_ssp, curpos_ssp, Transform2D::<i16, SchematicSpace, SchematicSpace>::identity())));
            },
            // moving
            (
                _, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::M, modifiers: _})
            ) => {
                state = SchematicState::Moving(None);
            },
            (
                SchematicState::Moving(Some((_ssp0, ssp1, _sst))),
                Event::Mouse(iced::mouse::Event::CursorMoved { .. })
            ) => {
                *ssp1 = curpos_ssp;
            },
            (
                SchematicState::Moving(Some((_ssp0, _ssp1, sst))), 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::R, modifiers: _})
            ) => {
                *sst = sst.then(&transforms::SST_CWR);
            },
            (
                SchematicState::Moving(mut opt_pts),
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            ) => {
                if let Some((ssp0, ssp1, vvt)) = &mut opt_pts {
                    self.move_selected(SchematicState::move_transform(ssp0, ssp1, vvt));
                    self.prune_nets();
                    state = SchematicState::Idle;
                    clear_passive = true;
                } else {
                    let ssp: euclid::Point2D<_, _> = curpos_ssp;
                    let sst = Transform2D::identity();
                    state = SchematicState::Moving(Some((ssp, ssp, sst)));
                }
            },
            // esc
            (
                st, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::Escape, modifiers: _})
            ) => {
                match st {
                    SchematicState::Idle => {
                        self.clear_selected();
                        clear_passive = true;
                    }
                    _ => {
                        state = SchematicState::Idle;
                    }
                }
            },
            // delete
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::Delete, modifiers: _})
            ) => {
                self.delete_selected();
                clear_passive = true;
            },
            // cycle
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::C, modifiers: _})
            ) => {
                ret = self.tentative_next_by_vspoint(curpos_ssp);
            },
            // test
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::T, modifiers: _})
            ) => {
                self.key_test();
            },
            // dc op
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::Space, modifiers: _})
            ) => {
                self.key_test();
                clear_passive = true;
            },
            _ => {},
        }
        self.state = state;
        (ret, clear_passive)
    }
}