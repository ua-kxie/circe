use std::rc::Rc;

use super::devicetype::{DeviceType, Port, Graphics};

use euclid::{Size2D, Transform2D, Vector2D, Angle};
use iced::{widget::canvas::{Frame, Stroke, stroke, LineCap, path::Builder, self, LineDash}, Color, Size};

use crate::{
    schematic::nets::{Selectable, Drawable},
    transforms::{
        SSPoint, VSBox, SSBox, VSPoint, VCTransform, Point, ViewportSpace, SchematicSpace
    }, 
};


#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeviceInstance {
    transform: euclid::Transform2D<i16, SchematicSpace, SchematicSpace>,
    device_type: Rc<DeviceType>,
    instance_bounds: SSBox,
    pub selected: bool,
    pub tentative: bool,
}

impl DeviceInstance {
    pub fn ports_ssp(&self) -> Vec<SSPoint> {
        self.device_type.get_ports().iter().map(|p| self.transform.transform_point(p.offset)).collect()
    }   
    pub fn ports_occupy_ssp(&self, ssp: SSPoint) -> bool {
        for p in self.device_type.get_ports() {
            if self.transform.transform_point(p.offset) == ssp {
                return true;
            }
        }
        return false;
    }
    fn stroke_bounds(&self, vct: VCTransform, frame: &mut Frame, stroke: Stroke) {
        let mut path_builder = Builder::new();
        let vsb = self.instance_bounds.cast().cast_unit();
        let csb = vct.outer_transformed_box(&vsb);
        let size = Size::new(csb.width(), csb.height());
        path_builder.rectangle(Point::from(csb.min).into(), size);
        frame.stroke(&path_builder.build(), stroke);    
    }
    fn stroke_symbol(&self, vct_composite: VCTransform, frame: &mut Frame, stroke: Stroke) {
        // let mut path_builder = Builder::new();
        for v1 in &self.device_type.get_graphics().pts {
            // there's a bug where dashed stroke can draw a solid line across a move
            // path_builder.move_to(Point::from(vct_composite.transform_point(v1[0])).into());
            let mut path_builder = Builder::new();
            for v0 in v1 {
                path_builder.line_to(Point::from(vct_composite.transform_point(*v0)).into());
            }
            frame.stroke(&path_builder.build(), stroke.clone());
        }
    }
    pub fn bounds(&self) -> &SSBox {
        &self.instance_bounds
    }
    pub fn set_translation(&mut self, v: SSPoint) {
        self.transform.m31 = v.x;
        self.transform.m32 = v.y;
        self.instance_bounds = self.transform.outer_transformed_box(&self.device_type.get_bounds());
    }
    pub fn pre_translate(&mut self, ssv: Vector2D<i16, SchematicSpace>) {
        self.transform = self.transform.pre_translate(ssv);
        self.instance_bounds = self.transform.outer_transformed_box(&self.device_type.get_bounds()); //self.device_type.as_ref().get_bounds().cast().cast_unit()
    }
    pub fn rotate(&mut self, cw: bool) {
        if cw {
            self.transform = self.transform.cast::<f32>().pre_rotate(Angle::frac_pi_2()).cast();
        } else {
            self.transform = self.transform.cast::<f32>().pre_rotate(-Angle::frac_pi_2()).cast();
        }
        self.instance_bounds = self.transform.cast().outer_transformed_box(&self.device_type.get_bounds().cast().cast_unit());
    }
    pub fn new_gnd(dt: Rc<DeviceType>) -> Self {
        let bds = SSBox::from_points([dt.get_bounds().min, dt.get_bounds().max]);
        DeviceInstance { 
            transform: Transform2D::identity(), 
            device_type: dt, 
            instance_bounds: bds,
            selected: false,
            tentative: false,
        }
    }
    
    pub fn new_res(ssp: SSPoint, dt: Rc<DeviceType>) -> Self {
        let bds = SSBox::from_points([dt.get_bounds().min, dt.get_bounds().max]);
        let mut d = DeviceInstance { 
            transform: Transform2D::identity(), 
            device_type: dt, 
            instance_bounds: bds,
            selected: false,
            tentative: false,
        };
        d.set_translation(ssp);
        d
    }
}

impl Selectable for DeviceInstance {
    fn collision_by_vsp(&self, curpos_vsp: VSPoint) -> bool {
        self.instance_bounds.contains(curpos_vsp.round().cast().cast_unit())
    }

    fn contained_by_vsb(&self, _selbox: VSBox) -> bool {
        todo!()
    }

    fn collision_by_vsb(&self, _selbox: VSBox) -> bool {
        todo!()
    }
}
const STROKE_WIDTH: f32 = 0.1;

impl Drawable for DeviceInstance {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.0),
            style: stroke::Style::Solid(Color::from_rgb(0.0, 0.8, 0.0)),
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        let vct_composite = self.transform.cast()
        .with_destination::<ViewportSpace>()
        .with_source::<ViewportSpace>()
        .then(&vct);
        // self.stroke_bounds(vct, frame, stroke.clone());
        self.stroke_symbol(vct_composite, frame, stroke.clone());
        for p in self.device_type.get_ports() {
            p.draw_persistent(vct_composite, vcscale, frame)
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
        // self.stroke_ports(vct, frame, stroke.clone());
        let vct_composite = self.transform.cast()
        .with_destination::<ViewportSpace>()
        .with_source::<ViewportSpace>()
        .then(&vct);
        self.stroke_symbol(vct_composite, frame, stroke.clone());
        for p in self.device_type.get_ports() {
            p.draw_selected(vct_composite, vcscale, frame)
        }
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let stroke = Stroke {
            width: (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 1.) / 2.0,
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.5)),
            line_cap: LineCap::Butt,
            line_dash: LineDash{segments: &[3. * (STROKE_WIDTH * vcscale).max(STROKE_WIDTH * 2.0)], offset: 0},
            ..Stroke::default()
        };
        let vct_composite = self.transform.cast()
        .with_destination::<ViewportSpace>()
        .with_source::<ViewportSpace>()
        .then(&vct);
        self.stroke_bounds(vct, frame, stroke.clone());
        self.stroke_symbol(vct_composite, frame, stroke.clone());
        for p in self.device_type.get_ports() {
            p.draw_preview(vct_composite, vcscale, frame)
        }
    }
}