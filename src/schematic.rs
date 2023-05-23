mod nets;
mod devices;

use std::sync::Arc;

pub use nets::{Selectable, Drawable, Nets, graph::{NetsGraph, NetsGraphExt, NetEdge, NetVertex}};
use crate::transforms::{VSPoint, SSPoint, VCTransform, VSBox, Point};
use devices::DeviceInstance;

use iced::{widget::canvas::{
    Frame, path::Builder, self,
}, Size, Color};
use self::devices::Devices;

#[derive(Clone, Debug)]
pub enum BaseElement {
    NetEdge(NetEdge),
    Device(Arc<DeviceInstance>),
}

#[derive(Clone, Debug)]
pub enum SchematicState {
    Wiring(Option<(Box<NetsGraph>, SSPoint)>),
    Idle,
    DevicePlacement(DeviceInstance),
    Selecting(VSBox),
}

impl Default for SchematicState {
    fn default() -> Self {
        SchematicState::Idle
    }
}

#[derive(Default)]
pub struct Schematic {
    net: Box<Nets>,
    devices: Devices,
    pub state: SchematicState,

    tentatives: Vec<BaseElement>,

    curpos: Option<(VSPoint, SSPoint)>,

    selskip: usize,
}

impl Schematic {
    pub fn tentatives_by_vsbox(&mut self, vsb: &VSBox) {
        self.tentatives.clear();
        let vsb_p = VSBox::from_points([vsb.min, vsb.max]);
        for d in self.devices.iter() {
            if d.bounds().intersects(&vsb_p) {
                self.tentatives.push(BaseElement::Device(d.clone()));
            }
        }
        for e in self.net.persistent.0.all_edges() {
            if vsb_p.contains(e.0.0.cast().cast_unit()) || vsb_p.contains(e.1.0.cast().cast_unit()) {
                self.tentatives.push(BaseElement::NetEdge(e.2.clone()));
            }
        }
    }
    pub fn tentative_by_vspoint(&mut self, vsp: VSPoint, skip: &mut usize) {
        self.tentatives.clear();
        if let Some(be) = self.selectable(vsp, skip) {
            self.tentatives.push(be);
        }
    }
    fn tentatives_to_selected(&mut self) {
        for be in &self.tentatives {
            match be {
                BaseElement::NetEdge(e) => {
                    self.net.select_edge(e.clone());
                },
                BaseElement::Device(d) => {
                    d.set_select();
                }
            }
        }
    }
    pub fn curpos_ssp(&self) -> Option<SSPoint> {
        self.curpos.map(|tup| tup.1)
    }
    pub fn left_click_down(&mut self) {
        let opt_ssp = self.curpos;
        match &mut self.state {
            SchematicState::Wiring(opt_ws) => {
                if let Some((_vsp, ssp)) = opt_ssp {
                    let mut new_ws = None;
                    if let Some((g, prev_ssp)) = opt_ws {  // subsequent click
                        if ssp == *prev_ssp { 
                        } else if self.net.persistent.occupies_ssp(ssp) {
                            self.net.as_mut().persistent.merge(g.as_ref());
                            new_ws = None;
                        } else {
                            self.net.as_mut().persistent.merge(g.as_ref());
                            new_ws = Some((Box::<NetsGraph>::default(), ssp));
                        }
                    } else {  // first click
                        new_ws = Some((Box::<NetsGraph>::default(), ssp));
                    }
                    *opt_ws = new_ws;
                }
            },
            SchematicState::Idle => {
                self.tentatives_to_selected();
                if let Some((vsp, _ssp)) = opt_ssp {
                    self.state = SchematicState::Selecting(VSBox::new(vsp, vsp));
                }

            },
            SchematicState::DevicePlacement(di) => {
                self.devices.push(di.clone());
                self.state = SchematicState::Idle;
            },
            SchematicState::Selecting(_) => {},
        };
    }
    pub fn left_click_up(&mut self) {
        match &mut self.state {
            SchematicState::Selecting(vsb) => {
                self.tentatives_to_selected();
                self.state = SchematicState::Idle;
            },
            _ => {},
        };
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
        };
        self.state = tmpst;
        self.curpos = opt_curpos;
    }

    pub fn draw_active(
        &self, 
        vct: VCTransform,
        vcscale: f32,
        frame: &mut Frame, 
    ) {  // draw elements which may need to be redrawn at any event
        for be in &self.tentatives {
            match be {
                BaseElement::NetEdge(e) => e.draw_preview(vct, vcscale, frame),
                BaseElement::Device(d) => d.draw_preview(vct, vcscale, frame),
            }
        }
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
                let mut path_builder = Builder::new();
                let csb = vct.outer_transformed_box(&vsb);
                let size = Size::new(csb.width(), csb.height());
                frame.fill_rectangle(Point::from(csb.min).into(), size, f);
            },
        }
    }

    pub fn draw_passive(
        &self, 
        vct: VCTransform,
        vcscale: f32,
        frame: &mut Frame, 
    ) {  // draw elements which may need to be redrawn at any event
        self.net.persistent.draw_persistent(vct, vcscale, frame);
        self.net.persistent.draw_selected(vct, vcscale, frame);
        self.devices.draw_persistent(vct, vcscale, frame);
        self.devices.draw_selected(vct, vcscale, frame);
    }

    pub fn bounding_box(&self) -> VSBox {
        let bbn = VSBox::from_points(self.net.persistent.0.nodes().map(|x| x.0.cast().cast_unit()));
        let bbi = self.devices.bounding_box();
        bbn.union(&bbi)
    }

    fn selectable(&self, curpos_vsp: VSPoint, skip: &mut usize) -> Option<BaseElement> {
        loop {
            let mut count = 0;
            for e in self.net.persistent.0.all_edges() {
                if e.2.collision_by_vsp(curpos_vsp) {
                    count += 1;
                    if count > *skip {
                        *skip = count;
                        return Some(BaseElement::NetEdge(e.2.clone()));
                    }
                }
            }
            for d in self.devices.iter() {
                if d.collision_by_vsp(curpos_vsp) {
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
            self.net.delete_selected_from_persistent();
            self.devices.delete_selected();
        }
    }
    pub fn clear_selected(&mut self) {
        match &mut self.state {
            SchematicState::Wiring(_) => {
                self.state = SchematicState::Idle;
            },
            SchematicState::DevicePlacement(_) => {
                self.state = SchematicState::Idle;
            },
            _ => {
                self.net.clear_selected();
                self.devices.clear_selected();
            }
        }
    }
    pub fn enter_wiring_mode(&mut self) {
        self.state = SchematicState::Wiring(None);
    }
    pub fn select_next_by_vspoint(&mut self) {
        if let Some((vsp, _ssp)) = self.curpos {
            self.tentatives.clear();
            let mut skip = self.selskip;
            if let Some(be) = self.selectable(vsp, &mut skip) {
                self.selskip = skip;
                self.tentatives.push(be);
            }
        }
    }
    pub fn key_r(&mut self) {
        match &mut self.state {
            SchematicState::Idle => {
                self.state = SchematicState::DevicePlacement(self.devices.place_res(self.curpos_ssp().unwrap_or(SSPoint::origin())));
            },
            SchematicState::Wiring(_) => {},
            SchematicState::DevicePlacement(di) => {
                di.rotate(true);
            },
            SchematicState::Selecting(_) => {},
        }
    }
    pub fn key_test(&mut self) {
        self.net.tt();
    }
    pub fn select_ini(&mut self) {
        if let Some((vsp, _)) = self.curpos {
            self.state = SchematicState::Selecting(VSBox::new(vsp, vsp))
        }
    }
    pub fn select_fin(&mut self) {
        if let SchematicState::Selecting(_vsb) = self.state {
            self.tentatives_to_selected();
            self.state = SchematicState::Idle;
        }
    }
}

