//! device strokes in device designer
//!

use std::{cell::RefCell, rc::Rc};

use iced::{
    widget::canvas::{path::Builder, stroke, Frame, LineCap, LineDash, Stroke},
    Color,
};

use crate::{
    schematic::interactable::{Interactable, Interactive},
    transforms::{Point, SSBox, SSPoint, SSVec, VCTransform},
    viewport::Drawable,
};

const STROKE_WIDTH: f32 = 0.8;

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
    pt0: SSPoint,
    pt1: SSPoint,
    pub interactable: Interactable,
}

impl Linear {
    pub fn new(ssp0: SSPoint, ssp1: SSPoint) -> Self {
        Linear {
            pt0: ssp0,
            pt1: ssp1,
            interactable: Interactable {
                bounds: SSBox::from_points([ssp0, ssp1]),
            },
        }
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
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.5)),
            line_cap: LineCap::Butt,
            line_dash: LineDash {
                segments: &[3. * (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.0)],
                offset: 0,
            },
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        path_builder.line_to(Point::from(vct.transform_point(self.pt0.cast().cast_unit())).into());
        path_builder.line_to(Point::from(vct.transform_point(self.pt1.cast().cast_unit())).into());
        frame.stroke(&path_builder.build(), stroke.clone());
    }
}

impl Interactive for Linear {
    fn transform(&mut self, sst: crate::transforms::SSTransform) {
        self.pt0 = sst.transform_point(self.pt0);
        self.pt1 = sst.transform_point(self.pt1);
        self.interactable = Interactable {
            bounds: SSBox::from_points([self.pt0, self.pt1]),
        }
    }
}

struct Ellipse {
    center: SSPoint,
    radii: SSVec,
}
