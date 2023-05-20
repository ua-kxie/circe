mod nets;
mod devices;

use std::sync::Arc;

pub use nets::{Selectable, Drawable, Nets, graph::{NetsGraph, NetsGraphExt, NetEdge, NetVertex}};
use crate::transforms::{VSPoint, SSPoint, VCTransform, VSBox};
use devices::DeviceInstance;

use iced::widget::canvas::{
    Frame,
};
use self::devices::Devices;

#[derive(Clone, Debug)]
pub enum BaseElement {
    NetEdge(NetEdge),
    NetVertex(NetVertex),
    Device(Arc<DeviceInstance>),
}

#[derive(Clone, Debug)]
pub enum SchematicState {
    Wiring(Option<(Box<NetsGraph>, SSPoint)>),
    Idle(Option<BaseElement>),
    DevicePlacement(DeviceInstance),
}

impl Default for SchematicState {
    fn default() -> Self {
        SchematicState::Idle(None)
    }
}

#[derive(Default)]
pub struct Schematic {
    net: Box<Nets>,
    devices: Devices,
    pub state: SchematicState,

    curpos: Option<(VSPoint, SSPoint)>,

    selskip: usize,
}

impl Schematic {
    pub fn curpos_ssp(&self) -> Option<SSPoint> {
        self.curpos.map(|tup| tup.1)
    }
    pub fn left_click(&mut self, ssp: SSPoint) {
        match &mut self.state {
            SchematicState::Wiring(opt_ws) => {
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
            },
            SchematicState::Idle(opt_be) => {
                if let Some(be) = opt_be {
                    match be {
                        BaseElement::NetEdge(e) => {
                            self.net.selected.0.add_edge(NetVertex(e.0), NetVertex(e.1), *e);
                        },
                        BaseElement::NetVertex(v) => {
                            self.net.selected.0.add_node(*v);
                        },
                        BaseElement::Device(d) => {
                            d.toggle_select();
                        }
                    }
                }
            },
            SchematicState::DevicePlacement(di) => {
                self.devices.push(di.clone());
                self.state = SchematicState::Idle(None);
            },
        };
    }
    pub fn curpos_update(&mut self, opt_curpos: Option<(VSPoint, SSPoint)>) {
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
            SchematicState::Idle(opt_be) => {
                if let Some((vsp, _)) = self.curpos {
                    let mut skip = self.selskip;
                    *opt_be = self.selectable(vsp, &mut skip);
                    self.selskip = skip;
                } else {
                    *opt_be = None;
                }
            },
            SchematicState::DevicePlacement(di) => {
                if let Some((_vsp, ssp)) = opt_curpos {
                    di.set_translation(ssp)
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
        match &self.state {
            SchematicState::Wiring(ws) => {
                if let Some((net, ..)) = ws {
                    net.as_ref().draw_preview(vct, vcscale, frame);
                }
            },
            SchematicState::Idle(opt_be) => {
                if let Some(be) = opt_be {
                    match be {
                        BaseElement::NetEdge(e) => e.draw_preview(vct, vcscale, frame),
                        BaseElement::NetVertex(v) => v.draw_preview(vct, vcscale, frame),
                        BaseElement::Device(d) => d.draw_preview(vct, vcscale, frame),
                    }
                }
            },
            SchematicState::DevicePlacement(di) => {
                di.draw_preview(vct, vcscale, frame);
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
        self.net.selected.draw_selected(vct, vcscale, frame);
        self.devices.draw_persistent(vct, vcscale, frame);
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
                        return Some(BaseElement::NetEdge(*e.2));
                    }
                }
            }
            for v in self.net.persistent.0.nodes() {
                if v.collision_by_vsp(curpos_vsp) {
                    count += 1;
                    if count > *skip {
                        *skip = count;
                        return Some(BaseElement::NetVertex(v));
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

    pub fn key_del(&mut self) {
        if let SchematicState::Idle(_) = self.state {
            self.net.delete_selected_from_persistent();
            self.devices.delete_selected();
        }
    }
    pub fn key_esc(&mut self) {
        match &mut self.state {
            SchematicState::Wiring(_) => {
                self.state = SchematicState::Idle(None);
            },
            SchematicState::Idle(_) => {
                self.net.selected.clear();
            },
            SchematicState::DevicePlacement(_) => {
                self.state = SchematicState::Idle(None);
            },
        }
    }
    pub fn key_wire(&mut self) {
        self.state = SchematicState::Wiring(None);
    }
    pub fn key_cycle(&mut self) {
        let mut tmpst = self.state.clone();
        if let SchematicState::Idle(opt_be) = &mut tmpst {
            if let Some((vsp, _)) = self.curpos {
                let mut skip = self.selskip;
                *opt_be = self.selectable(vsp, &mut skip);
                self.selskip = skip;
            } else {
                *opt_be = None;
            }
        }
        self.state = tmpst;
    }
    pub fn key_r(&mut self) {
        match &mut self.state {
            SchematicState::Idle(_) => {
                self.state = SchematicState::DevicePlacement(self.devices.place_res(self.curpos_ssp().unwrap_or(SSPoint::origin())));
            },
            SchematicState::Wiring(_) => {},
            SchematicState::DevicePlacement(di) => {
                di.rotate(true);
            },
        }
    }
    pub fn key_test(&mut self) {
        self.net.tt();
    }
}

