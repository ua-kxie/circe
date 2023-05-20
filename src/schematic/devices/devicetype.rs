use crate::{
    schematic::nets::{Selectable, Drawable},
    transforms::{
        SSVec, SSPoint, SSBox, VSBox, SSRect, VSPoint, VCTransform, Point, CanvasSpace, ViewportSpace, CSPoint, CSVec, VSRect, CSBox, CVTransform, VSVec
    }, 
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Port {
    pub name: &'static str,
    pub offset: SSVec,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graphics {
    pub pts: Vec<Vec<VSPoint>>
}

impl Graphics {
    pub fn new_gnd() -> Self {
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

    pub fn new_res() -> Self {
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
enum TypeEnum {
    // R(RDef),
    #[default] L,
    C,
    V,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeviceType {
    ports: Vec<Port>,
    bounds: SSBox,
    graphic: Graphics,
    type_enum: TypeEnum,  // R, L, C, V, etc.
}

impl DeviceType {
    pub fn get_ports(&self) -> &[Port] {
        &self.ports
    }
    pub fn get_bounds(&self) -> &SSBox {
        &self.bounds
    }
    pub fn get_graphics(&self) -> &Graphics {
        &self.graphic
    }
    pub fn get_type(&self) -> &TypeEnum {
        &self.type_enum
    }
    pub fn new_gnd() -> Self {
        Self {
            ports: vec![
                Port {name: "gnd", offset: SSVec::new(0, 2)}
            ],
            bounds: SSBox::new(SSPoint::new(-1, 2), SSPoint::new(1, -2)), 
            graphic: Graphics::new_gnd(),
            type_enum: TypeEnum::V, 
        }
    }
    pub fn new_res() -> Self {
        Self {
            ports: vec![
                Port {name: "+", offset: SSVec::new(0, 3)},
                Port {name: "-", offset: SSVec::new(0, -3)},
            ],
            bounds: SSBox::new(SSPoint::new(-2, 3), SSPoint::new(2, -3)), 
            graphic: Graphics::new_res(),
            type_enum: TypeEnum::C, 
        }
    }
}
