//! Ports, where wires go to get attached.

use iced::{
    widget::canvas::{self, path::Builder, stroke, LineCap, Stroke},
    Color, Size,
};
use std::{cell::RefCell, rc::Rc};
use crate::{
    transforms::{Point, SSBox, SSPoint, VCTransform, VSBox, VSVec, SSVec},
    viewport::Drawable, schematic::interactable::{Interactive, Interactable},
};

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
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct Port {
    /// the name of a port (necessary?)
    pub name: String,
    /// the offset of the port - position of the port relative to the device center
    pub offset: SSPoint,
    /// interactable only in effect in device designer
    pub interactable: Interactable,
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

impl Interactive for Port {
    fn transform(&mut self, sst: crate::transforms::SSTransform) {
        self.offset = sst.transform_point(self.offset);
        self.interactable = Interactable{bounds: SSBox::new(self.offset, self.offset + SSVec::new(1, 1))}
    }
}