//! device strokes in device designer
//!

use std::{cell::RefCell, rc::Rc};

use euclid::approxeq::ApproxEq;
use iced::{
    widget::canvas::{
        path::{Arc, Builder},
        stroke, Frame, LineCap, LineDash, Stroke,
    },
    Color,
};

use crate::{
    schematic::interactable::{Interactable, Interactive},
    transforms::{Point, SSBox, SSPoint, VCTransform, VSBox, VSPoint, VSVec},
    viewport::Drawable,
};

/// width of the stroke
const STROKE_WIDTH: f32 = 0.1;

/// newtype wrapper for `Rc<RefCell<Linear>>`
#[derive(Debug, Clone)]
pub struct RcRLinear(pub Rc<RefCell<Linear>>);

impl RcRLinear {
    pub fn new(l: Linear) -> Self {
        Self(Rc::new(RefCell::new(l)))
    }
}

#[derive(Debug, Clone)]
pub struct Linear {
    pt0: VSPoint,
    pt1: VSPoint,
    pub interactable: Interactable,
}

impl Linear {
    pub fn new(vsp0: VSPoint, vsp1: VSPoint) -> Self {
        Linear {
            pt0: vsp0,
            pt1: vsp1,
            interactable: Interactable {
                bounds: VSBox::from_points([vsp0, vsp1]),
            },
        }
    }
    pub fn pts(&self) -> (VSPoint, VSPoint) {
        (self.pt0, self.pt1)
    }
}

impl Drawable for Linear {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.0),
            style: stroke::Style::Solid(Color::from_rgb(0.0, 0.8, 0.0)),
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        path_builder.line_to(Point::from(vct.transform_point(self.pt0.cast().cast_unit())).into());
        path_builder.line_to(Point::from(vct.transform_point(self.pt1.cast().cast_unit())).into());
        frame.stroke(&path_builder.build(), stroke.clone());
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.) / 2.0,
            style: stroke::Style::Solid(Color::from_rgb(1.0, 0.8, 0.0)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        path_builder.line_to(Point::from(vct.transform_point(self.pt0.cast().cast_unit())).into());
        path_builder.line_to(Point::from(vct.transform_point(self.pt1.cast().cast_unit())).into());
        frame.stroke(&path_builder.build(), stroke.clone());
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 1.) / 2.0,
            style: stroke::Style::Solid(Color::from_rgba(1.0, 1.0, 0.5, 0.2)),
            line_cap: LineCap::Butt,
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        path_builder.line_to(Point::from(vct.transform_point(self.pt0)).into());
        path_builder.line_to(Point::from(vct.transform_point(self.pt1)).into());
        let built_path = path_builder.build();
        frame.stroke(&built_path, stroke);

        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 1.) / 2.0,
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.5)),
            line_cap: LineCap::Butt,
            line_dash: LineDash {
                segments: &[3. * (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.0)],
                offset: 0,
            },
            ..Stroke::default()
        };
        frame.stroke(&built_path, stroke);
    }
}

impl Interactive for Linear {
    fn transform(&mut self, vvt: crate::transforms::VVTransform) {
        self.pt0 = vvt.transform_point(self.pt0);
        self.pt1 = vvt.transform_point(self.pt1);
        self.interactable = Interactable {
            bounds: VSBox::from_points([self.pt0, self.pt1]),
        }
    }
}

/// newtype wrapper for `Rc<RefCell<Bounds>>`
#[derive(Debug, Clone)]
pub struct RcRCirArc(pub Rc<RefCell<CirArc>>);

impl RcRCirArc {
    pub fn new(b: CirArc) -> Self {
        Self(Rc::new(RefCell::new(b)))
    }
}
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CirArc {
    vsp0: VSPoint,
    vsp1: VSPoint,
    center: VSPoint,
    radius: f32,
    // start_angle: f32,
    // end_angle: f32,
    pub interactable: Interactable,
}

impl CirArc {
    pub fn from_triplet(vsp_center: VSPoint, vsp0: VSPoint, vsp1: VSPoint) -> Self {
        let radius = (vsp0 - vsp_center).length();
        let p0 = VSPoint::new(vsp_center.x - radius, vsp_center.y - radius);
        let p1 = VSPoint::new(vsp_center.x + radius, vsp_center.y + radius);
        CirArc {
            center: vsp_center,
            vsp0,
            vsp1,
            radius,
            interactable: Interactable::new(VSBox::from_points([p0, p1])),
        }
    }
    pub fn pts(&self) -> (VSPoint, VSPoint, VSPoint) {
        (self.center, self.vsp0, self.vsp1)
    }
    pub fn build_path(&self, vct: VCTransform, vcscale: f32, path_builder: &mut Builder) {
        if self.vsp0.approx_eq(&self.vsp1) {
            // render as circle
            path_builder.circle(
                Point::from(vct.transform_point(self.center)).into(),
                self.radius * vcscale,
            )
        } else {
            let start_angle_raw = vct
                .transform_vector(self.vsp0 - self.center)
                .angle_from_x_axis()
                .radians;
            let end_angle_raw = vct
                .transform_vector(self.vsp1 - self.center)
                .angle_from_x_axis()
                .radians;
            let start_angle = if start_angle_raw.is_finite() {
                start_angle_raw
            } else {
                0.0
            };
            let end_angle = if end_angle_raw.is_finite() {
                end_angle_raw
            } else {
                start_angle
            };
            path_builder.arc(Arc {
                center: Point::from(vct.transform_point(self.center)).into(),
                radius: self.radius * vcscale,
                start_angle,
                end_angle,
            })
        }
    }
}

impl Drawable for CirArc {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.0),
            style: stroke::Style::Solid(Color::from_rgb(0.0, 0.8, 0.0)),
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        self.build_path(vct, vcscale, &mut path_builder);
        frame.stroke(&path_builder.build(), stroke);
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.) / 2.0,
            style: stroke::Style::Solid(Color::from_rgb(1.0, 0.8, 0.0)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        self.build_path(vct, vcscale, &mut path_builder);
        frame.stroke(&path_builder.build(), stroke);
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 1.) / 2.0,
            style: stroke::Style::Solid(Color::from_rgba(1.0, 1.0, 0.5, 0.2)),
            line_cap: LineCap::Butt,
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        self.build_path(vct, vcscale, &mut path_builder);
        frame.stroke(&path_builder.build(), stroke);
    }
}

impl Interactive for CirArc {
    fn transform(&mut self, vvt: crate::transforms::VVTransform) {
        self.center = vvt.transform_point(self.center);
        self.vsp0 = vvt.transform_point(self.vsp0);
        self.vsp1 = vvt.transform_point(self.vsp1);
        let p0 = VSPoint::new(self.center.x - self.radius, self.center.y - self.radius);
        let p1 = VSPoint::new(self.center.x + self.radius, self.center.y + self.radius);
        self.interactable = Interactable {
            bounds: VSBox::from_points([p0, p1]),
        }
    }
}

/// newtype wrapper for `Rc<RefCell<Bounds>>`
#[derive(Debug, Clone)]
pub struct RcRBounds(pub Rc<RefCell<Bounds>>);

impl RcRBounds {
    pub fn new(b: Bounds) -> Self {
        Self(Rc::new(RefCell::new(b)))
    }
}

#[derive(Debug, Clone)]
pub struct Bounds {
    ssb: SSBox,
    pub interactable: Interactable,
}

impl Bounds {
    pub fn new(ssb: SSBox) -> Self {
        Bounds {
            ssb,
            interactable: Interactable::new(ssb.cast().cast_unit()),
        }
    }
    pub fn pts(&self) -> (SSPoint, SSPoint) {
        (self.ssb.min, self.ssb.max)
    }
}

impl Drawable for Bounds {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.0),
            style: stroke::Style::Solid(Color::from_rgba(0.8, 0.8, 0.8, 0.2)),
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        let cbox = vct.outer_transformed_box(&self.ssb.cast().cast_unit());
        let csize = cbox.max - cbox.min;
        let iced_size = iced::Size::from([csize.x, csize.y]);
        path_builder.rectangle(Point::from(cbox.min).into(), iced_size);
        frame.stroke(&path_builder.build(), stroke);
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.0) / 2.0,
            style: stroke::Style::Solid(Color::from_rgba(1.0, 1.0, 1.0, 0.8)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        let cbox = vct.outer_transformed_box(&self.ssb.cast().cast_unit());
        let csize = cbox.max - cbox.min;
        let iced_size = iced::Size::from([csize.x, csize.y]);
        path_builder.rectangle(Point::from(cbox.min).into(), iced_size);
        frame.stroke(&path_builder.build(), stroke);
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.0) / 2.0,
            style: stroke::Style::Solid(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
            line_cap: LineCap::Butt,
            line_dash: LineDash {
                segments: &[3. * (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.0)],
                offset: 0,
            },
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        let cbox = vct.outer_transformed_box(&self.ssb.cast().cast_unit());
        let csize = cbox.max - cbox.min;
        let iced_size = iced::Size::from([csize.x, csize.y]);
        path_builder.rectangle(Point::from(cbox.min).into(), iced_size);
        frame.stroke(&path_builder.build(), stroke);
    }
}

impl Interactive for Bounds {
    fn transform(&mut self, vvt: crate::transforms::VVTransform) {
        self.ssb = vvt
            .outer_transformed_box(&self.ssb.cast().cast_unit())
            .round()
            .cast()
            .cast_unit();
        self.interactable = Interactable {
            bounds: self.ssb.cast().cast_unit(),
        }
    }
}
