//! Ports, where wires go to get attached.

use crate::{
    schematic::interactable::{Interactable, Interactive},
    transforms::{vvt_to_sst, Point, SSPoint, VCTransform, VSBox, VSPoint, VSVec, VVTransform},
    viewport::Drawable,
};
use iced::{
    widget::canvas::{self, path::Builder, stroke, LineCap, Stroke},
    Color, Size,
};
use std::{cell::RefCell, rc::Rc};

const STROKE_WIDTH: f32 = 0.1;

/// newtype wrapper for `Rc<RefCell<Device>>`. Hashes by memory address.
#[derive(Debug, Clone)]
pub struct RcRPort(pub Rc<RefCell<Port>>);

impl RcRPort {
    pub fn new(p: Port) -> Self {
        Self(Rc::new(RefCell::new(p)))
    }
}

/// ports for devices, where wires may be connected
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Port {
    /// the name of a port (necessary?)
    pub name: String,
    /// the offset of the port - position of the port relative to the device center
    pub offset: SSPoint,
    /// interactable only in effect in device designer
    pub interactable: Interactable,
}

impl PartialEq for Port {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.offset == other.offset
    }
}

impl Eq for Port {}

impl std::hash::Hash for Port {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.offset.hash(state);
    }
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
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 1.0),
            style: stroke::Style::Solid(Color::from_rgb(0.8, 0.8, 0.0)),
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
            width: (STROKE_WIDTH * vcscale / 2.0).max(STROKE_WIDTH * 0.5),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.2)),
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

impl Interactive for Port {
    fn transform(&mut self, vvt: VVTransform) {
        self.offset = vvt_to_sst(vvt).transform_point(self.offset);
        let offset_vsp: VSPoint = self.offset.cast().cast_unit();
        self.interactable = Interactable {
            bounds: VSBox::from_points([offset_vsp, offset_vsp]),
        }
    }
}
