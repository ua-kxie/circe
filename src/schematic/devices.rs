// ex: Vgnd0 net1 0 0
// device Id, net at port, ground net '0', device voltage 0

use std::sync::Arc;

use euclid::{Size2D, Transform2D, Vector2D, Angle};
use iced::{widget::canvas::{Frame, Stroke, stroke, LineCap, path::Builder, self}, Color, Size};

use crate::{
    schematic::nets::{Selectable, Drawable},
    transforms::{
        SSVec, SSPoint, SSBox, VSBox, SSRect, VSPoint, VCTransform, Point, CanvasSpace, ViewportSpace, CSPoint, CSVec, VSRect, CSBox, CVTransform
    }, 
};

pub struct Devices {
    devices_vec: Vec<DeviceInstance>,

    res: Arc<DeviceType>,
}

impl Default for Devices {
    fn default() -> Self {
        let mut ret = Devices::new();
        ret.place_res();
        ret
    }
}

impl Drawable for Devices {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.devices_vec {
            d.draw_persistent(vct, vcscale, frame);
        }
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.devices_vec {
            d.draw_selected(vct, vcscale, frame);
        }
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.devices_vec {
            d.draw_preview(vct, vcscale, frame);
        }
    }
}

impl Devices {
    fn new() -> Self {
        Devices { devices_vec: vec![], res: Arc::new(DeviceType::new_res()) }
    }
    fn place_res(&mut self) {
        self.devices_vec.push(DeviceInstance::new_res(self.res.clone()))
    }
}

#[derive(Debug)]
struct Port {
    name: &'static str,
    offset: SSVec,
}

struct Graphics {
    pts: Vec<Vec<VSPoint>>
}

impl Graphics {
    fn new_gnd() -> Self {
        Self {
            pts: vec![
                vec![
                    VSPoint::new(0., 2.),
                    VSPoint::new(0., -1.)
                ],
                vec![
                    VSPoint::new(0., -2.),
                    VSPoint::new(1., -1.),
                    VSPoint::new(-1., -1.),
                    VSPoint::new(0., -2.),
                ],
            ]
        }
    }

    fn new_res() -> Self {
        Self {
            pts: vec![
                vec![
                    VSPoint::new(0., 3.),
                    VSPoint::new(0., -3.),
                ],
                vec![
                    VSPoint::new(-1., 2.),
                    VSPoint::new(-1., -2.),
                    VSPoint::new(1., -2.),
                    VSPoint::new(1., 2.),
                    VSPoint::new(-1., 2.),
                ],
            ]
        }
    }
}

// struct RawDef {
//     raw_definition: String,
// }

// impl RawDef {
//     fn definition(&self) -> &str {
//         &self.raw_definition
//     }
// }

// struct ValueDef {
//     value: f32,
// }

// impl ValueDef {
//     fn definition(&self) -> &str {
//         &self.value.to_string()
//     }
// }

// enum RDef {
//     Raw(RawDef),
//     Value(ValueDef),
// }

enum TypeEnum {
    // R(RDef),
    L,
    C,
    V,
}

pub struct DeviceType {
    ports: Vec<Port>,
    bounds: SSRect,
    graphic: Graphics,
    type_enum: TypeEnum,  // R, L, C, V, etc.
}

impl DeviceType {
    fn new_gnd() -> Self {
        Self {
            ports: vec![
                Port {name: "gnd", offset: SSVec::new(0, 2)}
            ],
            bounds: SSRect::new(SSPoint::origin(), Size2D::new(2, 4)), 
            graphic: Graphics::new_gnd(),
            type_enum: TypeEnum::V, 
        }
    }
    fn new_res() -> Self {
        Self {
            ports: vec![
                Port {name: "+", offset: SSVec::new(0, 3)},
                Port {name: "-", offset: SSVec::new(0, -3)},
            ],
            bounds: SSRect::new(SSPoint::origin(), Size2D::new(4, 6)), 
            graphic: Graphics::new_res(),
            type_enum: TypeEnum::C, 
        }
    }


}

pub struct DeviceInstance {
    transform: euclid::Transform2D<f32, ViewportSpace, ViewportSpace>,
    device_type: Arc<DeviceType>,
}

impl DeviceInstance {
    pub fn new_gnd(dt: Arc<DeviceType>) -> Self {
        DeviceInstance { 
            transform: Transform2D::identity(), 
            device_type: dt, 
        }
    }
    
    pub fn new_res(dt: Arc<DeviceType>) -> Self {
        DeviceInstance { 
            transform: Transform2D::identity(), 
            device_type: dt, 
        }
    }
}

impl Selectable for DeviceInstance {
    fn collision_by_vsp(&self, curpos_vsp: VSPoint) -> bool {
        self.device_type.bounds.to_box2d().cast().cast_unit().contains(curpos_vsp)
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
        let rect: VSRect = VSRect::new((port.offset.cast().cast_unit() - Vector2D::new(dim/2.0, dim/2.0)).to_point(), Size2D::new(dim, dim));

        let csrect = vct.outer_transformed_rect(&rect);
        
        let top_left = csrect.to_box2d().min;
        let size = Size::new(csrect.width(), csrect.height());
        frame.fill_rectangle(Point::from(top_left).into(), size, f.clone());
    }
}

impl Drawable for DeviceInstance {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let vct = self.transform.then(&vct);
        let solder_dia = 0.1;
        let wire_stroke = Stroke {
            width: (solder_dia * vcscale).max(solder_dia * 2.0),
            style: stroke::Style::Solid(Color::from_rgb(0.0, 0.8, 0.0)),
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        draw_with(&self.device_type.graphic, &self.device_type.ports, vct, frame, wire_stroke);
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let solder_dia = 0.3;
        let wire_stroke = Stroke {
            width: (solder_dia * vcscale).max(solder_dia * 20.),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 0.8, 0.0)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        draw_with(&self.device_type.graphic, &self.device_type.ports, vct, frame, wire_stroke);
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let solder_dia = 0.3;
        let wire_stroke = Stroke {
            width: (solder_dia * vcscale).max(solder_dia * 20.),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.5)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        draw_with(&self.device_type.graphic, &self.device_type.ports, vct, frame, wire_stroke);
    }
}