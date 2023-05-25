pub mod graph;
use std::cell::Cell;

use euclid::Vector2D;
pub use graph::Nets;

use petgraph::algo::tarjan_scc;

use crate::transforms::{VSPoint, VSBox, VCTransform, SchematicSpace};
use iced::widget::canvas::Frame;

use flagset::flags;

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

