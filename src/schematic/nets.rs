pub mod graph;
use std::cell::Cell;

use euclid::Vector2D;
use graph::NetsGraph;

use petgraph::algo::tarjan_scc;

use crate::transforms::{VSPoint, VSBox, VCTransform, SchematicSpace};
use iced::widget::canvas::Frame;

use flagset::flags;

use self::graph::NetsGraphExt;

use super::{NetEdge, NetVertex};

pub trait Selectable {
    // collision with point, selection box
    fn collision_by_vsp(&self, curpos_vsp: VSPoint) -> bool;
    fn contained_by_vsb(&self, selbox: VSBox) -> bool;
    fn collision_by_vsb(&self, selbox: VSBox) -> bool;
}

pub trait Drawable {
    const SOLDER_DIAMETER: f32 = 0.25;
    const WIRE_WIDTH: f32 = 0.05;
    const ZOOM_THRESHOLD: f32 = 5.0;
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
}

flags! {
    enum DrawState: u8 {
        Persistent,
        Selected,
        Preview,
    }
}

#[derive(Default)]
pub struct Nets {
    pub persistent: Box<NetsGraph>,
    // pub selected: NetsGraph,
}

impl Nets {
    pub fn move_selected(&mut self, ssv: Vector2D<i16, SchematicSpace>) {
        let mut tmp = vec![];
        for e in self.persistent.0.all_edges().filter(|e| e.2.2.get()) {
            tmp.push((e.0, e.1));
        }
        for e in tmp {
            self.persistent.0.remove_edge(e.0, e.1);
            let (ssp0, ssp1) = (e.0.0 + ssv, e.1.0 + ssv);
            self.persistent.0.add_edge(NetVertex(ssp0), NetVertex(ssp1), NetEdge(ssp0, ssp1, Cell::new(false)));
        }
        self.persistent.prune();
    }
    pub fn draw_selected_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for e in self.persistent.0.all_edges().filter(|e| e.2.2.get()) {
            e.2.draw_preview(vct, vcscale, frame);
        }
    }
    pub fn tt(&self) {
        let a = tarjan_scc(&self.persistent.0);  // this finds the unconnected components 
    }
    pub fn clear_selected(&self) {
        for e in self.persistent.0.all_edges() {
            e.2.2.set(false);
        }
    }
    pub fn select_edge(&mut self, e: NetEdge) {
        e.2.set(true);
        self.persistent.0.add_edge(NetVertex(e.0), NetVertex(e.1), e.clone());
    }
    pub fn delete_selected_from_persistent(&mut self) {
        let mut tmp = vec![];
        for e in self.persistent.0.all_edges().filter(|e| e.2.2.get()) {
            tmp.push((e.0, e.1));
        }
        for e in tmp {
            self.persistent.0.remove_edge(e.0, e.1);
        }
        self.persistent.prune();
    }
}
