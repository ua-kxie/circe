use euclid::{Size2D, Transform2D, Vector2D, Angle};
use iced::{widget::canvas::{Frame, Stroke, stroke, LineCap, path::Builder, self}, Color, Size};

// ex: Vgnd0 net1 0 0
// device Id, net at port, ground net '0', device voltage 0
use crate::{schematic::nets::{Selectable, Drawable},
    transforms::{
        SSVec, SSPoint, SSBox, VSBox, SSRect, VSPoint, VCTransform, Point, CanvasSpace, ViewportSpace, CSPoint, CSVec, VSRect, CSBox, CVTransform
    }, 
    viewport::Viewport
};

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

pub struct Device {
    transform: euclid::Transform2D<f32, ViewportSpace, ViewportSpace>,
    ports: Vec<Port>,
    bounds: SSRect,
    graphic: Graphics,
}

impl Default for Device {
    fn default() -> Self {
        Device::new_res(SSPoint::origin())
    }
}

impl Device {
    pub fn new_gnd(ssp: SSPoint) -> Self {
        Device { 
            transform: Transform2D::identity(), 
            ports: vec![
                Port {name: "gnd", offset: SSVec::new(0, 2)}
            ],
            bounds: SSRect::new(SSPoint::origin(), Size2D::new(2, 4)), 
            graphic: Graphics::new_gnd() 
        }
    }
    
    pub fn new_res(ssp: SSPoint) -> Self {
        Device { 
            transform: Transform2D::identity().then_rotate(Angle::frac_pi_2()), 
            ports: vec![
                Port {name: "+", offset: SSVec::new(0, 3)},
                Port {name: "-", offset: SSVec::new(0, -3)},
            ],
            bounds: SSRect::new(SSPoint::origin(), Size2D::new(4, 6)), 
            graphic: Graphics::new_res() 
        }
    }
}

impl Selectable for Device {
    fn collision_by_vsp(&self, curpos_vsp: VSPoint) -> bool {
        self.bounds.to_box2d().cast().cast_unit().contains(curpos_vsp)
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

impl Drawable for Device {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let vct = self.transform.then(&vct);
        let solder_dia = 0.1;
        let wire_stroke = Stroke {
            width: (solder_dia * vcscale).max(solder_dia * 2.0),
            style: stroke::Style::Solid(Color::from_rgb(0.0, 0.8, 0.0)),
            line_cap: LineCap::Square,
            ..Stroke::default()
        };
        draw_with(&self.graphic, &self.ports, vct, frame, wire_stroke);
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let solder_dia = 0.3;
        let wire_stroke = Stroke {
            width: (solder_dia * vcscale).max(solder_dia * 20.),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 0.8, 0.0)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        draw_with(&self.graphic, &self.ports, vct, frame, wire_stroke);
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let solder_dia = 0.3;
        let wire_stroke = Stroke {
            width: (solder_dia * vcscale).max(solder_dia * 20.),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.5)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        draw_with(&self.graphic, &self.ports, vct, frame, wire_stroke);
    }
}