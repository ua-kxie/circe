mod nets;
mod devices;

use std::{rc::Rc, cell::RefCell};

pub use nets::{Selectable, Drawable, Nets, graph::{NetEdge, NetVertex}};
use crate::transforms::{VSPoint, SSPoint, VCTransform, VSBox, Point};
use devices::DeviceInstance;

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
    Moving(SSPoint, SSPoint),
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
    pub fn tentative_next_by_vspoint(&mut self, curpos_vsp: VSPoint) {
        let mut skip = self.selskip;
        self.tentative_by_vspoint(curpos_vsp, &mut skip)
    }
    fn tentatives_to_selected(&mut self) {
        self.nets.tentatives_to_selected();
        self.devices.tentatives_to_selected();
    }
    fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.nets.occupies_ssp(ssp) || self.devices.occupies_ssp(ssp)
    }
    pub fn left_click_down(&mut self, curpos_vsp: VSPoint) {
        let mut state = self.state.clone();
        match &mut state {
            SchematicState::Wiring(opt_ws) => {
                let ssp = curpos_vsp.round().cast().cast_unit();
                let mut new_ws = None;
                if let Some((g, prev_ssp)) = opt_ws {  // subsequent click
                    if ssp == *prev_ssp { 
                    } else if self.occupies_ssp(ssp) {
                        self.nets.merge(g.as_ref());
                        new_ws = None;
                    } else {
                        self.nets.merge(g.as_ref());
                        new_ws = Some((Box::<Nets>::default(), ssp));
                    }
                } else {  // first click
                    new_ws = Some((Box::<Nets>::default(), ssp));
                }
                // *opt_ws = new_ws;
                self.state = SchematicState::Wiring(new_ws);
            },
            SchematicState::Idle => {
                self.tentatives_to_selected();
                self.state = SchematicState::Selecting(VSBox::new(curpos_vsp, curpos_vsp));
            },
            SchematicState::DevicePlacement(di) => {
                self.devices.push(di.clone());
                self.state = SchematicState::Idle;
            },
            SchematicState::Selecting(_) => {},
            SchematicState::Moving(ssp0, ssp1) => {
                let ssv = *ssp1 - *ssp0;
                self.nets.move_selected(ssv);
                self.devices.move_selected(ssv);
                self.state = SchematicState::Idle;
            },
        };
    }
    pub fn left_click_up(&mut self) {
        if let SchematicState::Selecting(_) = self.state {
            self.tentatives_to_selected();
            self.state = SchematicState::Idle;
        }
    }
    pub fn curpos_update(&mut self, opt_curpos: Option<(VSPoint, SSPoint)>) {
        if let Some((vsp, _ssp)) = opt_curpos {
            let mut skip = self.selskip.saturating_sub(1);
            self.tentative_by_vspoint(vsp, &mut skip);
            self.selskip = skip;
        }
        let mut tmpst = self.state.clone();
        match &mut tmpst {
            SchematicState::Wiring(opt_ws) => {
                if let Some((g, prev_ssp)) = opt_ws {
                    g.as_mut().clear();
                    if let Some((_vsp, ssp)) = opt_curpos {
                        g.route(*prev_ssp, ssp);
                    }
                }
            },
            SchematicState::Idle => {
            },
            SchematicState::DevicePlacement(di) => {
                if let Some((_vsp, ssp)) = opt_curpos {
                    di.set_translation(ssp);
                }
            },
            SchematicState::Selecting(vsb) => {
                if let Some((vsp, _ssp)) = opt_curpos {
                    vsb.max = vsp;
                    self.tentatives_by_vsbox(&vsb);
                }
            },
            SchematicState::Moving(_, ssp1) => {
                if let Some((_vsp, ssp)) = opt_curpos {
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
                    style: canvas::Style::Solid(if vsb.height() > 0. {Color::from_rgba(1., 1., 0., 0.1)} else {Color::from_rgba(0., 1., 1., 0.1)}),
                    ..canvas::Fill::default()
                };
                let csb = vct.outer_transformed_box(&vsb);
                let size = Size::new(csb.width(), csb.height());
                frame.fill_rectangle(Point::from(csb.min).into(), size, f);
            },
            SchematicState::Moving(ssp0, ssp1) => {
                let vct_c = vct.pre_translate((*ssp1 - *ssp0).cast().cast_unit());
                self.nets.draw_selected_preview(vct_c, vcscale, frame);
                self.devices.draw_selected_preview(vct_c, vcscale, frame);
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
            self.nets.delete_selected_from_persistent();
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
    pub fn enter_wiring_mode(&mut self) {
        self.state = SchematicState::Wiring(None);
    }
    pub fn key_r(&mut self, curpos_ssp: SSPoint) {
        match &mut self.state {
            SchematicState::Idle => {
                self.state = SchematicState::DevicePlacement(self.devices.place_res(curpos_ssp));
            },
            SchematicState::DevicePlacement(di) => {
                di.rotate(true);
            },
            _ => {},
        }
    }
    pub fn key_test(&mut self) {
        self.nets.tt();
    }
    pub fn move_(&mut self, curpos_ssp: SSPoint) {
        self.state = SchematicState::Moving(curpos_ssp, curpos_ssp);
    }
}

