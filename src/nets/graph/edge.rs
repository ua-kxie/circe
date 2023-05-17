use crate::{
    transforms::{
        VSPoint, SSPoint, VSBox, SSBox, VCTransform, CVTransform, ViewportSpace, SchematicSpace
    }, 
    nets::{Drawable, DrawState, Selectable}
};
use euclid::{Point2D, Box2D, Vector2D};
use iced::{widget::canvas::{Frame, Path, Stroke, stroke, LineCap, LineDash}, Color};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NetEdge (pub SSPoint, pub SSPoint);

impl NetEdge {
    pub fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        let v = self.1 - self.0;
        if v.x == 0 {
            // anywhere on vertical line
            ssp.x == self.0.x && ssp.y < self.0.y.max(self.1.y) && ssp.y > self.0.y.min(self.1.y)
        } else if v.y == 0 {
            // anywhere on horizontal line
            ssp.y == self.0.y && ssp.x < self.0.x.max(self.1.x) && ssp.x > self.0.x.min(self.1.x)
        } else {  
            // either edge of oblique line
            ssp == self.0 || ssp == self.1
        }
    }
}

impl Selectable for NetEdge {
    fn collision_by_vsp(&self, curpos_vsp: VSPoint) -> bool {
        let v = self.1 - self.0;
        if v.x == 0 {
            let ab = VSBox::from_points([  // from pts instead of new to guarantee positive sized box
                self.0.cast().cast_unit(), 
                self.1.cast().cast_unit()
            ]).inflate(0.2, 0.);
            ab.contains(curpos_vsp)
        } else if v.y == 0 {
            let ab = VSBox::from_points([  // from pts instead of new to guarantee positive sized box
                self.0.cast().cast_unit(), 
                self.1.cast().cast_unit()
            ]).inflate(0., 0.2);
            ab.contains(curpos_vsp)
        } else {  // oblique line
            // find transform `t` to take stored geometry to unit horizontal line
            // should be a better/faster way todo
            let mut t = CVTransform::identity();
            let v1: Vector2D<f32, ViewportSpace> = v.cast().cast_unit();
            t = t.then_rotate(v1.angle_from_x_axis());
            t = t.then_translate(self.0.to_vector().cast().cast_unit());
            let t = t.inverse().unwrap();

            // transform curpos_vsp with A
            let p = t.transform_point(curpos_vsp);

            // check if resulting point is contained in box around horizontal line
            let bounds = Box2D::from_points([Point2D::from([v1.length()/10., 0.2]), Point2D::from([v1.length() - v1.length()/10., -0.2])]);
            bounds.contains(p)
        }
    }

    fn contained_by_vsb(&self, _selbox: VSBox) -> bool {
        todo!()
    }

    fn collision_by_vsb(&self, _selbox: VSBox) -> bool {
        todo!()
    }
}

fn draw_with(src: SSPoint, dst: SSPoint, vct: VCTransform, frame: &mut Frame, stroke: Stroke) {
    let psrcv = vct.transform_point(src.cast().cast_unit());
    let pdstv = vct.transform_point(dst.cast().cast_unit());
    let c = Path::line(
        iced::Point::from([psrcv.x, psrcv.y]),
        iced::Point::from([pdstv.x, pdstv.y]),
    );
    frame.stroke(&c, stroke);
}

impl Drawable for NetEdge {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let wire_width = 0.02;
        let wire_stroke = Stroke {
            width: (wire_width * vcscale).max(wire_width * 50.),
            style: stroke::Style::Solid(Color::from_rgb(0.0, 0.8, 1.0)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        draw_with(self.0, self.1, vct, frame, wire_stroke);
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let wire_width = 0.02;
        let wire_stroke = Stroke {
            width: (wire_width * vcscale).max(wire_width * 50.),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 0.8, 0.0)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        draw_with(self.0, self.1, vct, frame, wire_stroke);
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let wire_width = 0.02;
        let wire_stroke = Stroke {
            width: (wire_width * vcscale).max(wire_width * 50.),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.5)),
            line_cap: LineCap::Butt,
            line_dash: LineDash{segments: &[3. * (wire_width * vcscale).max(wire_width)], offset: 0},
            ..Stroke::default()
        };
        draw_with(self.0, self.1, vct, frame, wire_stroke);
    }
}