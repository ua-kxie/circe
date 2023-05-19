pub mod graph;
use graph::NetsGraph;

use petgraph::algo::tarjan_scc;

use crate::transforms::{VSPoint, VSBox, VCTransform};
use iced::widget::canvas::Frame;

use flagset::flags;

use self::graph::NetsGraphExt;

pub trait Selectable {
    // collision with point, selection box
    fn collision_by_vsp(&self, curpos_vsp: VSPoint) -> bool;
    fn contained_by_vsb(&self, selbox: VSBox) -> bool;
    fn collision_by_vsb(&self, selbox: VSBox) -> bool;
}

pub trait Drawable {
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
    pub persistent: NetsGraph,
    pub selected: NetsGraph,
}

impl Nets {
    pub fn tt(&self) {
        let a = tarjan_scc(&self.persistent.0);  // this finds the unconnected components 
    }
    // pub fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame, curpos_vsp: VSPoint) {
    //     for (_, _, edge) in self.persistent.0.all_edges() {
    //         if edge.collision_by_vsp(curpos_vsp) {
    //             edge.draw_preview(vct, vcscale, frame)
    //         }
    //     }
    //     for vertex in self.persistent.0.nodes() {
    //         if vertex.collision_by_vsp(curpos_vsp) {
    //             vertex.draw_preview(vct, vcscale, frame)
    //         }
    //     }
    // }
    pub fn delete_selected_from_persistent(&mut self) {
        // for v in self.selected.0.nodes() {
        //     self.persistent.0.remove_node(v);
        // }
        for e in self.selected.0.all_edges() {
            self.persistent.0.remove_edge(e.0, e.1);
        }
        self.persistent.prune();
        self.selected.clear();
    }
}
