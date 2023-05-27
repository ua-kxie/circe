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
        for e in self.nets.0.all_edges_mut() {
            if vsb_p.contains(e.0.0.cast().cast_unit()) || vsb_p.contains(e.1.0.cast().cast_unit()) {
                e.2.tentative = true;
            }
        }
    }
    pub fn tentative_by_vspoint(&mut self, vsp: VSPoint, skip: &mut usize) {
        self.clear_tentatives();
        if let Some(be) = self.selectable(vsp, skip) {
            match be {
                BaseElement::NetEdge(e) => {
                    let mut netedge = e.clone();
                    netedge.tentative = true;
                    self.nets.0.add_edge(NetVertex(e.src), NetVertex(e.dst), netedge);
                },
                BaseElement::Device(d) => {
                    d.borrow_mut().tentative = true;
                },
            }
        }
    }
    pub fn tentative_next_by_vspoint(&mut self, curpos_ssp: SSPoint) {
        let mut skip = self.selskip;
        self.tentative_by_vspoint(curpos_ssp.cast().cast_unit(), &mut skip)
    }
    fn tentatives_to_selected(&mut self) {
        self.nets.tentatives_to_selected();
        self.devices.tentatives_to_selected();
    }
    fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.nets.occupies_ssp(ssp) || self.devices.occupies_ssp(ssp)
    }
    pub fn left_click_up(&mut self) {
        if let SchematicState::Selecting(_) = self.state {
            self.tentatives_to_selected();
            self.state = SchematicState::Idle;
        }
    }
    pub fn curpos_update(&mut self, vsp: VSPoint, ssp: SSPoint) {
        let mut skip = self.selskip.saturating_sub(1);
        self.tentative_by_vspoint(vsp, &mut skip);
        self.selskip = skip;

        let mut tmpst = self.state.clone();
        match &mut tmpst {
            SchematicState::Wiring(opt_ws) => {
                if let Some((g, prev_ssp)) = opt_ws {
                    g.as_mut().clear();
                    g.route(*prev_ssp, ssp);
                }
            },
            SchematicState::Idle => {
            },
            SchematicState::DevicePlacement(di) => {
                di.set_translation(ssp);
            },
            SchematicState::Selecting(ssb) => {
                ssb.max = vsp.round().cast().cast_unit();
                self.tentatives_by_vsbox(&ssb.cast().cast_unit());
            },
            SchematicState::Moving(opt_pts) => {
                if let Some((_ssp0, ssp1)) = opt_pts {
                    *ssp1 = ssp;
                }
            },
        };
        self.state = tmpst;
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
            SchematicState::Wiring(ws) => {
                if let Some((net, ..)) = ws {
                    net.as_ref().draw_preview(vct, vcscale, frame);
                }
            },
            SchematicState::Idle => {
            },
            SchematicState::DevicePlacement(di) => {
                di.draw_preview(vct, vcscale, frame);
            },
            SchematicState::Selecting(vsb) => {
                let f = canvas::Fill {
                    style: canvas::Style::Solid(if vsb.height() > 0 {Color::from_rgba(1., 1., 0., 0.1)} else {Color::from_rgba(0., 1., 1., 0.1)}),
                    ..canvas::Fill::default()
                };
                let csb = vct.outer_transformed_box(&vsb.cast().cast_unit());
                let size = Size::new(csb.width(), csb.height());
                frame.fill_rectangle(Point::from(csb.min).into(), size, f);
            },
            SchematicState::Moving(opt_pts) => {
                if let Some((ssp0, ssp1)) = opt_pts {
                    let vct_c = vct.pre_translate((*ssp1 - *ssp0).cast().cast_unit());
                    self.nets.draw_selected_preview(vct_c, vcscale, frame);
                    self.devices.draw_selected_preview(vct_c, vcscale, frame);
                }
            },
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
        let bbn = VSBox::from_points(self.nets.0.nodes().map(|x| x.0.cast().cast_unit()));
        let bbi = self.devices.bounding_box();
        bbn.union(&bbi)
    }

    fn selectable(&self, curpos_vsp: VSPoint, skip: &mut usize) -> Option<BaseElement> {
        loop {
            let mut count = 0;
            for e in self.nets.0.all_edges() {
                if e.2.collision_by_vsp(curpos_vsp) {
                    count += 1;
                    if count > *skip {
                        *skip = count;
                        return Some(BaseElement::NetEdge(e.2.clone()));
                    }
                }
            }
            for d in self.devices.iter() {
                if d.borrow().collision_by_vsp(curpos_vsp) {
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
    pub fn esc(&mut self) {
        match &mut self.state {
            SchematicState::Idle => {
                self.clear_selected();
            }
            _ => {
                self.state = SchematicState::Idle;
            }
        }
    }
    pub fn key_test(&mut self) {
        self.nets.tt();
    }

    pub fn events_handler(
        &mut self, 
        event: iced::widget::canvas::Event, 
        curpos_ssp: SSPoint, 
    ) -> (bool, bool) {
        // let mut msg = None;
        let mut clear_active = false;
        let mut clear_passive = false;
        let mut state = self.state.clone();
        match (&mut state, event) {
            // wiring
            (
                _, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::M, modifiers})
            ) => {
                self.state = SchematicState::Wiring(None);
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
                self.state = SchematicState::Wiring(new_ws);
                clear_active = true;
                clear_passive = true;
            },
            // selecting
            (
                SchematicState::Idle, 
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            ) => {
                self.tentatives_to_selected();
                self.state = SchematicState::Selecting(SSBox::new(curpos_ssp, curpos_ssp));
            },
            (
                SchematicState::Selecting(ssb), 
                Event::Mouse(iced::mouse::Event::CursorMoved { .. })
            ) => {
                ssb.max = curpos_ssp;
                self.tentatives_by_vsbox(&ssb.cast().cast_unit());
            },
            // device placement
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::R, modifiers})
            ) => {
                self.state = SchematicState::DevicePlacement(self.devices.place_res(curpos_ssp));
                clear_active = true;
            },
            (
                SchematicState::DevicePlacement(di), 
                Event::Mouse(iced::mouse::Event::CursorMoved { .. })
            ) => {
                di.set_translation(curpos_ssp);
                clear_active = true;
            },
            (
                SchematicState::DevicePlacement(di), 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::R, modifiers})
            ) => {
                di.rotate(true);
                clear_active = true;
            },
            (
                SchematicState::DevicePlacement(di), 
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            ) => {
                self.devices.push(di.clone());
                self.state = SchematicState::Idle;
                clear_active = true;
                clear_passive = true;
            },
            // moving
            (
                _, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::M, modifiers})
            ) => {
                self.state = SchematicState::Moving(None);
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
                    self.state = SchematicState::Idle;
                } else {
                    let ssp: euclid::Point2D<_, _> = curpos_ssp;
                    self.state = SchematicState::Moving(Some((ssp, ssp)));
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
                        clear_active = true;
                    }
                    _ => {
                        self.state = SchematicState::Idle;
                        clear_active = true;
                    }
                }
            },
            // delete
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::Escape, modifiers})
            ) => {
                self.delete_selected();
                clear_active = true;
                clear_passive = true;
            },
            // cycle
            (
                SchematicState::Idle, 
                Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::Escape, modifiers})
            ) => {
                self.tentative_next_by_vspoint(curpos_ssp);
                clear_active = true;
            },
            _ => {},
        }
        (clear_active, clear_passive)
    }
}