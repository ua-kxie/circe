mod nets;
mod devices;
mod interactable;

use std::{collections::HashSet};

use euclid::Vector2D;
pub use nets::{Selectable, Drawable, Nets, graph::{NetEdge, NetVertex}};
use crate::transforms::{VSPoint, SSPoint, VCTransform, VSBox, Point, SSBox, SchematicSpace, CSPoint};
use iced::widget::canvas::{event::Event, path::Builder, Stroke, LineCap};
use iced::{widget::canvas::{
    Frame, self,
}, Size, Color};
use self::{devices::{Devices, RcRDevice}, interactable::Interactive};


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
    DevicePlacement(RcRDevice),
    Selecting(SSBox),
    Moving(Option<(SSPoint, SSPoint)>),
}

impl Default for SchematicState {
    fn default() -> Self {
        SchematicState::Idle
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
    fn clear_selected(&mut self) {
        self.selected.clear();
    }
    fn clear_tentatives(&mut self) {
        self.devices.clear_tentatives();
        self.nets.clear_tentatives();
    }
    pub fn tentatives_by_ssbox(&mut self, ssb: &SSBox) {
        self.clear_tentatives();
        let ssb_p = SSBox::from_points([ssb.min, ssb.max]);
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
                    d.0.borrow_mut().set_tentative();
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
            SchematicState::DevicePlacement(d) => {
                d.0.borrow().draw_preview(vct, vcscale, frame);
            }
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
                frame.stroke(&path_builder.build(), stroke)
            },
            SchematicState::Moving(Some((ssp0, ssp1))) => {
                let vct_c = vct.pre_translate((*ssp1 - *ssp0).cast().cast_unit());
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

    fn selectable(&self, curpos_ssp: SSPoint, skip: &mut usize) -> Option<BaseElement> {
        loop {
            let mut count = 0;
            for e in self.nets.graph.all_edges() {
                if e.2.collision_by_ssp(curpos_ssp) {
                    count += 1;
                    if count > *skip {
                        *skip = count;
                        return Some(BaseElement::NetEdge(e.2.clone()));
                    }
                }
            }
            if let Some(d) = self.devices.selectable(curpos_ssp, skip, &mut count) {
                return Some(BaseElement::Device(d));
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
        self.nets.tt();
    }
    fn prune_nets(&mut self) {
        self.nets.prune(self.devices.ports_ssp());
    }
    fn move_selected(&mut self, ssv: Vector2D<i16, SchematicSpace>) {
        let selected = self.selected.clone();
        self.selected.clear();
        for be in selected {
            match be {
                BaseElement::NetEdge(e) => {
                    self.nets.translate(e, ssv);
                }
                BaseElement::Device(d) => {
                    d.0.borrow_mut().translate(ssv);
                }
            }
        }
    }

    pub fn events_handler(
        &mut self, 
        event: Event, 
        curpos_vsp: VSPoint,
        curpos_ssp: SSPoint, 
    ) -> (Option<crate::Msg>, bool) {
        let mut msg = None;
        let mut clear_passive = false;

        if let Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = event {
            let mut skip = self.selskip.saturating_sub(1);
            let s = self.tentative_by_sspoint(curpos_ssp, &mut skip);
            self.selskip = skip;
            msg = Some(crate::Msg::NetName(s));
        }

        let mut state = self.state.clone();
        match (&mut state, event) {
            // wiring
            (
                _, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::W, modifiers})
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
                self.tentatives_by_ssbox(&ssb);
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
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::R, modifiers})
            ) => {
                let d = self.devices.new_res();
                d.0.borrow_mut().set_translation(curpos_ssp);
                state = SchematicState::DevicePlacement(d);
            },
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::G, modifiers})
            ) => {
                let d = self.devices.new_gnd();
                d.0.borrow_mut().set_translation(curpos_ssp);
                state = SchematicState::DevicePlacement(d);
            },
            (
                SchematicState::DevicePlacement(d), 
                Event::Mouse(iced::mouse::Event::CursorMoved { .. })
            ) => {
                d.0.borrow_mut().set_translation(curpos_ssp);
            },
            (
                SchematicState::DevicePlacement(d), 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::R, modifiers})
            ) => {
                d.0.borrow_mut().rotate(true);
            },
            (
                SchematicState::DevicePlacement(di), 
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            ) => {
                self.devices.insert(di.clone());
                self.prune_nets();
                state = SchematicState::Idle;
                clear_passive = true;
            },
            // moving
            (
                _, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::M, modifiers})
            ) => {
                state = SchematicState::Moving(None);
            },
            (
                SchematicState::Moving(Some((_ssp0, ssp1))),
                Event::Mouse(iced::mouse::Event::CursorMoved { .. })
            ) => {
                *ssp1 = curpos_ssp;
            },
            (
                SchematicState::Moving(mut opt_pts),
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            ) => {
                if let Some((ssp0, ssp1)) = &mut opt_pts {
                    let ssv = *ssp1 - *ssp0;
                    self.move_selected(ssv);
                    self.prune_nets();
                    state = SchematicState::Idle;
                    clear_passive = true;
                } else {
                    let ssp: euclid::Point2D<_, _> = curpos_ssp;
                    state = SchematicState::Moving(Some((ssp, ssp)));
                }
            },
            // esc
            (
                st, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::Escape, modifiers})
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
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::Delete, modifiers})
            ) => {
                self.delete_selected();
                clear_passive = true;
            },
            // cycle
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::C, modifiers})
            ) => {
                let s = self.tentative_next_by_vspoint(curpos_ssp);
                msg = Some(crate::Msg::NetName(s));
            },
            _ => {},
        }
        self.state = state;
        (msg, clear_passive)
    }
}