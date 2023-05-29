mod nets;
mod devices;

use std::{rc::Rc, cell::RefCell};

pub use nets::{Selectable, Drawable, Nets, graph::{NetEdge, NetVertex}};
use crate::transforms::{VSPoint, SSPoint, VCTransform, VSBox, Point, SSBox};
use devices::DeviceInstance;
use iced::widget::canvas::event::{self, Event};
use iced::{widget::canvas::{
    Frame, self,
}, Size, Color};
use self::devices::Devices;

#[derive(Clone)]
pub enum BaseElement {
    NetEdge(NetEdge),
    Device(Rc<RefCell<DeviceInstance>>),
}

#[derive(Clone, Debug)]
pub enum SchematicState {
    Wiring(Option<(Box<Nets>, SSPoint)>),
    Idle,
    DevicePlacement(DeviceInstance),
    Selecting(VSBox),
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
}

impl Schematic {
    fn clear_selected(&mut self) {
        self.devices.clear_selected();
        self.nets.clear_selected();
    }
    fn clear_tentatives(&mut self) {
        self.devices.clear_tentatives();
        self.nets.clear_tentatives();
    }
    pub fn tentatives_by_vsbox(&mut self, vsb: &VSBox) {
        self.clear_tentatives();
        let vsb_p = VSBox::from_points([vsb.min, vsb.max]);
        for d in self.devices.iter() {
            if d.borrow().bounds().cast().cast_unit().intersects(&vsb_p) {
                d.borrow_mut().tentative = true;
            }
        }
        for e in self.nets.graph.all_edges_mut() {
            if vsb_p.contains(e.0.0.cast().cast_unit()) || vsb_p.contains(e.1.0.cast().cast_unit()) {
                e.2.tentative = true;
            }
        }
    }
    pub fn tentative_by_sspoint(&mut self, ssp: SSPoint, skip: &mut usize) -> Option<String> {
        self.clear_tentatives();
        if let Some(be) = self.selectable(ssp, skip) {
            match be {
                BaseElement::NetEdge(e) => {
                    let netname = e.label.borrow().clone();
                    let mut netedge = e.clone();
                    netedge.tentative = true;
                    self.nets.graph.add_edge(NetVertex(e.src), NetVertex(e.dst), netedge);
                    Some(netname)
                },
                BaseElement::Device(d) => {
                    d.borrow_mut().tentative = true;
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
        self.nets.tentatives_to_selected();
        self.devices.tentatives_to_selected();
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
        self.nets.draw_preview(vct, vcscale, frame);
        self.devices.draw_preview(vct, vcscale, frame);

        match &self.state {
            SchematicState::Wiring(Some((net, ..))) => {
                net.as_ref().draw_preview(vct, vcscale, frame);
            },
            SchematicState::Idle => {
            },
            SchematicState::DevicePlacement(di) => {
                di.draw_preview(vct, vcscale, frame);
            },
            SchematicState::Selecting(vsb) => {
                let f = canvas::Fill {
                    style: canvas::Style::Solid(if vsb.height() > 0.0 {Color::from_rgba(1., 1., 0., 0.1)} else {Color::from_rgba(0., 1., 1., 0.1)}),
                    ..canvas::Fill::default()
                };
                let csb = vct.outer_transformed_box(&vsb.cast().cast_unit());
                let size = Size::new(csb.width(), csb.height());
                frame.fill_rectangle(Point::from(csb.min).into(), size, f);
            },
            SchematicState::Moving(Some((ssp0, ssp1))) => {
                let vct_c = vct.pre_translate((*ssp1 - *ssp0).cast().cast_unit());
                self.nets.draw_selected_preview(vct_c, vcscale, frame);
                self.devices.draw_selected_preview(vct_c, vcscale, frame);
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
        self.nets.draw_selected(vct, vcscale, frame);
        self.devices.draw_persistent(vct, vcscale, frame);
        self.devices.draw_selected(vct, vcscale, frame);
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
            for d in self.devices.iter() {
                if d.borrow().collision_by_ssp(curpos_ssp) {
                    count += 1;
                    if count > *skip {
                        *skip = count;
                        return Some(BaseElement::Device(d.clone()));
                    }
                }
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
            self.nets.delete_selected_from_persistent(self.devices.ports_ssp());
            self.devices.delete_selected();
        }
    }
    pub fn key_test(&mut self) {
        self.nets.tt();
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
                self.tentatives_to_selected();
                state = SchematicState::Selecting(VSBox::new(curpos_vsp, curpos_vsp));
            },
            (
                SchematicState::Selecting(vsb), 
                Event::Mouse(iced::mouse::Event::CursorMoved { .. })
            ) => {
                vsb.max = curpos_vsp;
                self.tentatives_by_vsbox(&vsb);
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
                state = SchematicState::DevicePlacement(self.devices.place_res(curpos_ssp));
            },
            (
                SchematicState::DevicePlacement(di), 
                Event::Mouse(iced::mouse::Event::CursorMoved { .. })
            ) => {
                di.set_translation(curpos_ssp);
            },
            (
                SchematicState::DevicePlacement(di), 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::R, modifiers})
            ) => {
                di.rotate(true);
            },
            (
                SchematicState::DevicePlacement(di), 
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            ) => {
                self.devices.push(di.clone());
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
                    self.nets.move_selected(ssv);
                    self.devices.move_selected(ssv);
                    self.nets.prune(self.devices.ports_ssp());
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