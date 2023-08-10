//! device strokes in device designer
//!

use std::{cell::RefCell, rc::Rc};

use iced::{
    widget::canvas::{path::Builder, stroke, Frame, LineCap, LineDash, Stroke},
    Color,
};

use crate::{
    schematic::interactable::{Interactable, Interactive},
    transforms::{Point, SSBox, VCTransform, VSBox, VSPoint, VSVec, SSPoint},
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
#[allow(unused)]
struct Ellipse {
    center: VSPoint,
    radii: VSVec,
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
