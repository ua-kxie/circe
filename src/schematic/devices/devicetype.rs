//! device type. Resistors are a distinct type from capacitors, etc.

use iced::{
    widget::canvas::{self, path::Builder, stroke, Frame, LineCap, LineDash, Stroke},
    Color, Size,
};

use crate::{
    transforms::{Point, SSBox, SSPoint, VCTransform, VSBox, VSPoint, VSVec},
    viewport::Drawable,
};

pub mod gnd;
pub mod r;
pub mod v;

/// ports for devices, where wires may be connected
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct Port {
    /// the name of a port (necessary?)
    pub name: String,
    /// the offset of the port - position of the port relative to the device center
    pub offset: SSPoint,
}

impl Drawable for Port {
    fn draw_persistent(
        &self,
        vct: VCTransform,
        _vcscale: f32,
        frame: &mut iced::widget::canvas::Frame,
    ) {
        let f = canvas::Fill {
            style: canvas::Style::Solid(Color::from_rgba(1.0, 0.0, 0.0, 1.0)),
            ..canvas::Fill::default()
        };
        let dim = 0.4;
        let ssb = VSBox::new(
            self.offset.cast::<f32>().cast_unit() - VSVec::new(dim / 2.0, dim / 2.0),
            self.offset.cast::<f32>().cast_unit() + VSVec::new(dim / 2.0, dim / 2.0),
        );

        let csbox = vct.outer_transformed_box(&ssb);

        let top_left = csbox.min;
        let size = Size::new(csbox.width(), csbox.height());
        frame.fill_rectangle(Point::from(top_left).into(), size, f);
    }

    fn draw_selected(
        &self,
        vct: crate::transforms::VCTransform,
        vcscale: f32,
        frame: &mut iced::widget::canvas::Frame,
    ) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 1.),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.0)),
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        let dim = 0.4;
        let vsb = VSBox::new(
            self.offset.cast::<f32>().cast_unit() - VSVec::new(dim / 2.0, dim / 2.0),
            self.offset.cast::<f32>().cast_unit() + VSVec::new(dim / 2.0, dim / 2.0),
        );
        let csb = vct.outer_transformed_box(&vsb);
        let size = Size::new(csb.width(), csb.height());
        path_builder.rectangle(Point::from(csb.min).into(), size);
        frame.stroke(&path_builder.build(), stroke);
    }

    fn draw_preview(
        &self,
        vct: crate::transforms::VCTransform,
        vcscale: f32,
        frame: &mut iced::widget::canvas::Frame,
    ) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 1.),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.5)),
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        let dim = 0.4;
        let vsb = VSBox::new(
            self.offset.cast::<f32>().cast_unit() - VSVec::new(dim / 2.0, dim / 2.0),
            self.offset.cast::<f32>().cast_unit() + VSVec::new(dim / 2.0, dim / 2.0),
        );
        let csb = vct.outer_transformed_box(&vsb);
        let size = Size::new(csb.width(), csb.height());
        path_builder.rectangle(Point::from(csb.min).into(), size);
        frame.stroke(&path_builder.build(), stroke);
    }
}

const STROKE_WIDTH: f32 = 0.1;

/// graphical representation for devices
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Graphics {
    /// line is traced from point to point for each inner vector.
    pts: Vec<Vec<VSPoint>>,
    /// arbitrary number of circles (center, radius) to be drawn
    circles: Vec<(VSPoint, f32)>,
    /// arbitrary number of device ports
    ports: Vec<Port>,
    /// device bounds
    bounds: SSBox,
}
impl Graphics {
    pub fn bounds(&self) -> &SSBox {
        &self.bounds
    }
    pub fn ports(&self) -> &[Port] {
        &self.ports
    }
    pub fn stroke_bounds(&self, vct_composite: VCTransform, frame: &mut Frame, stroke: Stroke) {
        let mut path_builder = Builder::new();
        let vsb = self.bounds.cast().cast_unit();
        let csb = vct_composite.outer_transformed_box(&vsb);
        let size = Size::new(csb.width(), csb.height());
        path_builder.rectangle(Point::from(csb.min).into(), size);
        frame.stroke(&path_builder.build(), stroke);
    }
    pub fn stroke_symbol(
        &self,
        vct_composite: VCTransform,
        vcscale: f32,
        frame: &mut Frame,
        stroke: Stroke,
    ) {
        // let mut path_builder = Builder::new();
        for v1 in &self.pts {
            // there's a bug where dashed stroke can draw a solid line across a move
            // path_builder.move_to(Point::from(vct_composite.transform_point(v1[0])).into());
            let mut path_builder = Builder::new();
            for v0 in v1 {
                path_builder.line_to(Point::from(vct_composite.transform_point(*v0)).into());
            }
            frame.stroke(&path_builder.build(), stroke.clone());
        }
        let mut path_builder = Builder::new();
        for (p, r) in &self.circles {
            path_builder.circle(
                Point::from(vct_composite.transform_point(*p)).into(),
                *r * vcscale,
            );
        }
        frame.stroke(&path_builder.build(), stroke.clone());
    }
}
impl Drawable for Graphics {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.0),
            style: stroke::Style::Solid(Color::from_rgb(0.0, 0.8, 0.0)),
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        self.stroke_symbol(vct, vcscale, frame, stroke.clone());
        for p in &self.ports {
            p.draw_persistent(vct, vcscale, frame)
        }
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.) / 2.0,
            style: stroke::Style::Solid(Color::from_rgb(1.0, 0.8, 0.0)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        self.stroke_bounds(vct, frame, stroke.clone());
        self.stroke_symbol(vct, vcscale, frame, stroke.clone());
        for p in &self.ports {
            p.draw_selected(vct, vcscale, frame)
        }
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
        self.stroke_bounds(vct, frame, stroke.clone());
        self.stroke_symbol(vct, vcscale, frame, stroke.clone());
        for p in &self.ports {
            p.draw_preview(vct, vcscale, frame)
        }
    }
}

pub trait DeviceType {
    fn default_graphics() -> Graphics;
}

/// DeviceClass enumerates the various classes of devices. E.g. ground, resistor, voltage source... etc
#[derive(Debug, Clone)]
pub enum DeviceClass {
    Gnd(gnd::Gnd),
    R(r::R),
    V(v::V),
}
impl DeviceClass {
    /// sets the raw parameter of the device
    pub fn set_raw_param(&mut self, new: String) {
        match self {
            DeviceClass::R(x) => match &mut x.params {
                r::ParamR::Raw(y) => y.set(new),
                r::ParamR::Value(_) => {}
            },
            DeviceClass::Gnd(_) => {}
            DeviceClass::V(x) => match &mut x.params {
                v::ParamV::Raw(y) => y.set(new),
            },
        }
    }
    /// returns a reference to the device graphics
    pub fn graphics(&self) -> &'static Graphics {
        match self {
            DeviceClass::Gnd(x) => x.graphics,
            DeviceClass::R(x) => x.graphics,
            DeviceClass::V(x) => x.graphics,
        }
    }
    /// returns a summary of the device parameter for display on canvas
    pub fn param_summary(&self) -> String {
        match self {
            DeviceClass::Gnd(x) => x.params.summary(),
            DeviceClass::R(x) => x.params.summary(),
            DeviceClass::V(x) => x.params.summary(),
        }
    }
    /// returns the id prefix of the device class
    pub fn id_prefix(&self) -> &'static str {
        match self {
            DeviceClass::Gnd(_) => gnd::ID_PREFIX,
            DeviceClass::R(_) => r::ID_PREFIX,
            DeviceClass::V(_) => v::ID_PREFIX,
        }
    }
}
