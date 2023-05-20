use std::sync::Arc;

use super::devicetype::{DeviceType, Port, Graphics};

use euclid::{Size2D, Transform2D, Vector2D, Angle};
use iced::{widget::canvas::{Frame, Stroke, stroke, LineCap, path::Builder, self}, Color, Size};

use crate::{
    schematic::nets::{Selectable, Drawable},
    transforms::{
        SSPoint, VSBox, VSPoint, VCTransform, Point, ViewportSpace
    }, 
};

use std::cell::Cell;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeviceInstance {
    transform: euclid::Transform2D<i16, ViewportSpace, ViewportSpace>,
    device_type: Arc<DeviceType>,
    instance_bounds: VSBox,
    selected: Cell<bool>,
}

impl DeviceInstance {
    pub fn bounds(&self) -> &VSBox {
        &self.instance_bounds
    }
    pub fn toggle_select(&self) {
        self.selected.set(!self.selected.get());
    }
    pub fn selected(&self) -> bool {
        self.selected.get()
    }
    pub fn set_translation(&mut self, v: SSPoint) {
        self.transform.m31 = v.x;
        self.transform.m32 = v.y;
        self.instance_bounds = self.transform.cast().outer_transformed_box(&self.device_type.get_bounds().cast().cast_unit());
    }
    pub fn rotate(&mut self, cw: bool) {
        if cw {
            self.transform = self.transform.cast::<f32>().pre_rotate(Angle::frac_pi_2()).cast();
        } else {
            self.transform = self.transform.cast::<f32>().pre_rotate(-Angle::frac_pi_2()).cast();
        }
        self.instance_bounds = self.transform.cast().outer_transformed_box(&self.device_type.get_bounds().cast().cast_unit());
    }
    pub fn new_gnd(dt: Arc<DeviceType>) -> Self {
        let bds = VSBox::from_points([dt.get_bounds().min.cast().cast_unit(), dt.get_bounds().max.cast().cast_unit()]);
        DeviceInstance { 
            transform: Transform2D::identity(), 
            device_type: dt, 
            instance_bounds: bds,
            selected: Cell::new(false),
        }
    }
    
    pub fn new_res(ssp: SSPoint, dt: Arc<DeviceType>) -> Self {
        let bds = VSBox::from_points([dt.get_bounds().min.cast().cast_unit(), dt.get_bounds().max.cast().cast_unit()]);
        let mut d = DeviceInstance { 
            transform: Transform2D::identity(), 
            device_type: dt, 
            instance_bounds: bds,
            selected: Cell::new(false),
        };
        d.set_translation(ssp);
        d
    }
}

impl Selectable for DeviceInstance {
    fn collision_by_vsp(&self, curpos_vsp: VSPoint) -> bool {
        self.instance_bounds.contains(curpos_vsp)
    }

    fn contained_by_vsb(&self, _selbox: VSBox) -> bool {
        todo!()
    }

    fn collision_by_vsb(&self, _selbox: VSBox) -> bool {
        todo!()
    }
}

fn draw_with(graphics: &Graphics, ports: &[Port], vct: VCTransform, frame: &mut Frame, stroke: Stroke) {
    let mut path_builder = Builder::new();
    for v1 in &graphics.pts {
        path_builder.move_to(Point::from(vct.transform_point(v1[0])).into());
        for v0 in v1 {
            path_builder.line_to(Point::from(vct.transform_point(*v0)).into());
        }
    }
    frame.stroke(&path_builder.build(), stroke);

    let f = canvas::Fill {
        style: canvas::Style::Solid(Color::from_rgb(1.0, 0.0, 0.0)),
        ..canvas::Fill::default()
    };
    for port in ports {
        let dim = 0.4;
        let ssb = VSBox::new(
            (port.offset.cast::<f32>().cast_unit() - Vector2D::new(dim/2.0, dim/2.0)), 
            (port.offset.cast::<f32>().cast_unit() + Vector2D::new(dim/2.0, dim/2.0)), 
        );

        let csbox = vct.outer_transformed_box(&ssb);
        
        let top_left = csbox.min;
        let size = Size::new(csbox.width(), csbox.height());
        frame.fill_rectangle(Point::from(top_left).into(), size, f.clone());
    }
}

impl Drawable for DeviceInstance {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let vct_composite = self.transform.cast().then(&vct);
        let solder_dia = 0.1;
        let wire_stroke = Stroke {
            width: (solder_dia * vcscale).max(solder_dia * 2.0),
            style: stroke::Style::Solid(Color::from_rgb(0.0, 0.8, 0.0)),
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        draw_with(&self.device_type.get_graphics(), &self.device_type.get_ports(), vct_composite, frame, wire_stroke);
        
        if self.selected.get() {
            let solder_dia = 0.1;
            let stroke = Stroke {
                width: (solder_dia * vcscale).max(solder_dia * 2.),
                style: stroke::Style::Solid(Color::from_rgb(1.0, 0.8, 0.0)),
                line_cap: LineCap::Round,
                ..Stroke::default()
            };
            let mut path_builder = Builder::new();
            let vsb = self.instance_bounds;
            let csb = vct.outer_transformed_box(&vsb);
            let size = Size::new(csb.width(), csb.height());
            path_builder.rectangle(Point::from(csb.min).into(), size);
            frame.stroke(&path_builder.build(), stroke);    
        }
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        // let vct = self.transform.cast().then(&vct);
        // let solder_dia = 0.3;
        // let wire_stroke = Stroke {
        //     width: (solder_dia * vcscale).max(solder_dia * 2.),
        //     style: stroke::Style::Solid(Color::from_rgb(1.0, 0.8, 0.0)),
        //     line_cap: LineCap::Round,
        //     ..Stroke::default()
        // };
        // draw_with(&self.device_type.get_graphics(), &self.device_type.get_ports(), vct, frame, wire_stroke);
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let solder_dia = 0.1;
        let stroke = Stroke {
            width: (solder_dia * vcscale).max(solder_dia * 1.),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.5)),
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        let mut path_builder = Builder::new();
        let rect = self.instance_bounds;
        let rect = vct.outer_transformed_box(&rect.cast().cast_unit());
        let size = Size::new(rect.width(), rect.height());
        path_builder.rectangle(Point::from(rect.min).into(), size);
        frame.stroke(&path_builder.build(), stroke);    
    }
}