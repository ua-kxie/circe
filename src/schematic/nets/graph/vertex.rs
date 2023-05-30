/// petgraph vertices weight
/// in GraphMap, also serve as the keys
/// 
use std::cmp::Ordering;

use crate::{transforms::{VSPoint, SSPoint, VSBox, VCTransform}, schematic::nets::{Drawable, Selectable}};
use euclid::Box2D;
use iced::{widget::canvas::{Frame, Path, Stroke, stroke, LineCap}, Color};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NetVertex (pub SSPoint);

impl NetVertex {
    pub fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.0 == ssp
    }
}

impl PartialOrd for NetVertex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NetVertex {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0.x, self.0.y).cmp(&(other.0.x, other.0.y))
    }
}

impl Selectable for NetVertex {
    fn collision_by_ssp(&self, curpos_ssp: SSPoint) -> bool {
        false
    }
    fn collision_by_vsp(&self, curpos_vsp: VSPoint) -> bool {
        let bb: VSBox = Box2D::new(self.0, self.0).cast().cast_unit();
        bb.inflate(0.5, 0.5).contains(curpos_vsp)
    }

    fn contained_by_vsb(&self, _selbox: VSBox) -> bool {
        todo!()
    }

    fn collision_by_vsb(&self, _selbox: VSBox) -> bool {
        todo!()
    }
}

fn draw_with(ssp: SSPoint, vct: VCTransform, frame: &mut Frame, stroke: Stroke) {
    let p = vct.transform_point(ssp.cast().cast_unit());
    let p = iced::Point::from([p.x, p.y]);
    let c = Path::line(p, p,);
    frame.stroke(&c, stroke);
}
const SOLDER_DIAMETER: f32 = 0.25;
const WIRE_WIDTH: f32 = 0.05;
const ZOOM_THRESHOLD: f32 = 5.0;

impl Drawable for NetVertex {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let solder_dia = self::SOLDER_DIAMETER;
        let zoom_thshld = self::ZOOM_THRESHOLD;
        let wire_stroke = Stroke {
            width: (solder_dia * vcscale).max(solder_dia * zoom_thshld),
            style: stroke::Style::Solid(Color::from_rgb(0.0, 0.8, 1.0)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        draw_with(self.0, vct, frame, wire_stroke);
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let solder_dia = self::SOLDER_DIAMETER;
        let zoom_thshld = self::ZOOM_THRESHOLD;
        let wire_stroke = Stroke {
            width: (solder_dia * vcscale).max(solder_dia * zoom_thshld),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 0.8, 0.0)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        draw_with(self.0, vct, frame, wire_stroke);
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let solder_dia = self::SOLDER_DIAMETER;
        let zoom_thshld = self::ZOOM_THRESHOLD;
        let wire_stroke = Stroke {
            width: (solder_dia * vcscale).max(solder_dia * zoom_thshld),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.5)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        draw_with(self.0, vct, frame, wire_stroke);
    }
}