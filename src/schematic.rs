pub use crate::nets::{Selectable, Drawable, Nets, graph::{NetsGraph, NetsGraphExt, NetEdge, NetVertex}};
use iced::widget::Canvas;
use iced::widget::canvas::event::Event;
use iced::mouse::Event::*;
use iced::mouse;

use crate::transforms::{VSPoint, SSPoint, ViewportSpace, SchematicSpace, CSPoint, VCTransform, VSBox};

use iced::widget::canvas::{
    stroke, Cache, Cursor, LineCap, Path, Stroke, LineDash, Frame,
};
use iced::Color;

#[derive(Clone, Debug)]
pub enum BaseElement {
    NetEdge(NetEdge),
    NetVertex(NetVertex),
}

#[derive(Clone, Debug)]
pub enum SchematicState {
    Wiring(Option<(Box<NetsGraph>, SSPoint)>),
    Idle(Option<BaseElement>),
}

impl Default for SchematicState {
    fn default() -> Self {
        SchematicState::Idle(None)
    }
}

#[derive(Default)]
pub struct Schematic {
    net: Box<Nets>,
    pub state: SchematicState,

    curpos: Option<(VSPoint, SSPoint)>,

    selskip: usize,
}

impl Schematic {
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
                    }
                }
            },
        };
    }
    pub fn curpos_update(&mut self, opt_curpos: Option<(VSPoint, SSPoint)>) {
        let mut tmpst = self.state.clone();
        match &mut tmpst {
            SchematicState::Wiring(opt_ws) => {
                if let Some((g, prev_ssp)) = opt_ws {
                    g.as_mut().clear();
                    if let Some((vsp, ssp)) = opt_curpos {
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
                    }
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
        self.net.persistent.draw_persistent(vct, vcscale, frame);
        self.net.selected.draw_selected(vct, vcscale, frame);
    }

    pub fn bounding_box(&self) -> VSBox {
        VSBox::from_points(self.net.persistent.0.nodes().map(|x| x.0.cast().cast_unit()))
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
                dbg!(skip);
                *opt_be = self.selectable(vsp, &mut skip);
                dbg!(skip);
                self.selskip = skip;
            } else {
                *opt_be = None;
            }
        }
        self.state = tmpst;
    }
    pub fn key_test(&mut self) {
        self.net.tt();
    }
}

