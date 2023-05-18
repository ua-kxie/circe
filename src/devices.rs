use euclid::{Size2D, default::Transform2D};

// ex: Vgnd0 net1 0 0
// device Id, net at port, ground net '0', device voltage 0
use crate::transforms::{SSVec, SSPoint, SSBox, SSRect, VSPoint};

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
                    VSPoint::new(0., -2.),
                    VSPoint::new(0., 1.)
                ],
                vec![
                    VSPoint::new(0., 2.),
                    VSPoint::new(1., 1.),
                    VSPoint::new(-1., 1.),
                    VSPoint::new(0., 2.),
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

struct Device {
    transform: euclid::default::Transform2D<f32>,
    ports: Vec<Port>,
    bounds: SSRect,
    graphic: Graphics,
}

impl Device {
    fn new_gnd(ssp: SSPoint) -> Self {
        Device { 
            transform: Transform2D::identity(), 
            ports: vec![
                Port {name: "gnd", offset: SSVec::new(0, 2)}
            ],
            bounds: SSRect::new(SSPoint::origin(), Size2D::new(2, 4)), 
            graphic: Graphics::new_gnd() 
        }
    }
    
    fn new_res(ssp: SSPoint) -> Self {
        Device { 
            transform: Transform2D::identity(), 
            ports: vec![
                Port {name: "+", offset: SSVec::new(0, 3)},
                Port {name: "-", offset: SSVec::new(0, -3)},
            ],
            bounds: SSRect::new(SSPoint::origin(), Size2D::new(4, 6)), 
            graphic: Graphics::new_res() 
        }
    }
}