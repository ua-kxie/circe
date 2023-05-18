// ex: Vgnd0 net1 0 0
// device Id, net at port, ground net '0', device voltage 0
use crate::transforms::{SSVec, SSPoint, SSBox};

struct Port {
    name: &'static str,
    offset: SSVec,
}

struct Graphics {}

struct Device {
    position: SSPoint,
    ports: [Port; 1],
    bounds: SSBox,
    graphic: Graphics,
}

impl Device {
    fn new_gnd(ssp: SSPoint) -> Self {
        Device { 
            position: ssp, 
            ports: [
                Port {name: "gnd", offset: SSVec::new(0, 2)},
                ], 
            bounds: SSBox::new(SSPoint::new(-1, 2), SSPoint::new(1, -2)), 
            graphic: Graphics{} 
        }
    }
}