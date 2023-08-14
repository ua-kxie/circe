//! device definition for resistors (RXXXX)

use super::super::params;
use super::{Graphics, Port};
use crate::schematic::interactable::Interactable;
use crate::transforms::{SSBox, SSPoint, VSPoint};
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "R";

lazy_static! {
    static ref DEFAULT_GRAPHICS: Graphics = Graphics {
        pts: vec![
            vec![VSPoint::new(1.00, -0.25), VSPoint::new(-1.00, -0.75),],
            vec![VSPoint::new(-1.00, -0.75), VSPoint::new(1.00, -1.25),],
            vec![VSPoint::new(1.00, -1.25), VSPoint::new(-1.00, -1.75),],
            vec![VSPoint::new(0.00, -2.00), VSPoint::new(0.00, -3.00),],
            vec![VSPoint::new(-1.00, -1.75), VSPoint::new(0.00, -2.00),],
            vec![VSPoint::new(1.00, 1.75), VSPoint::new(-1.00, 1.25),],
            vec![VSPoint::new(1.00, 0.75), VSPoint::new(-1.00, 0.25),],
            vec![VSPoint::new(-1.00, 1.25), VSPoint::new(1.00, 0.75),],
            vec![VSPoint::new(0.00, 3.00), VSPoint::new(0.00, 2.00),],
            vec![VSPoint::new(0.00, 2.00), VSPoint::new(1.00, 1.75),],
            vec![VSPoint::new(-1.00, 0.25), VSPoint::new(1.00, -0.25),],
        ],
        cirarcs: vec![],
        ports: vec![
            Port {
                name: "0".to_string(),
                offset: SSPoint::new(0, 3),
                interactable: Interactable::default()
            },
            Port {
                name: "1".to_string(),
                offset: SSPoint::new(0, -3),
                interactable: Interactable::default()
            },
        ],
        bounds: SSBox::new(SSPoint::new(-2, -3), SSPoint::new(2, 3)),
    };
}
<<<<<<< HEAD

// lazy_static! {
//     static ref DEFAULT_GRAPHICS: Graphics = Graphics {
//         pts: vec![
//             vec![VSPoint::new(0., 3.), VSPoint::new(0., -3.),],
//             vec![
//                 VSPoint::new(-1., 2.),
//                 VSPoint::new(-1., -2.),
//                 VSPoint::new(1., -2.),
//                 VSPoint::new(1., 2.),
//                 VSPoint::new(-1., 2.),
//             ],
//         ],
//         circles: vec![],
//         ports: vec![
//             Port {
//                 name: "+".to_string(),
//                 offset: SSPoint::new(0, 3),
//                 interactable: Interactable::default(),
//             },
//             Port {
//                 name: "-".to_string(),
//                 offset: SSPoint::new(0, -3),
//                 interactable: Interactable::default(),
//             },
//         ],
//         bounds: SSBox::new(SSPoint::new(-2, 3), SSPoint::new(2, -3)),
//     };
// }
=======
>>>>>>> 9e8b2a7 (added nmos, pmos devices)

/// Enumerates the different ways to specifify parameters for a resistor
#[derive(Debug, Clone)]
pub enum ParamR {
    /// specify the spice line directly (after id and port connections)
    Raw(params::Raw),
}
impl Default for ParamR {
    fn default() -> Self {
        ParamR::Raw(params::Raw::new(String::from("1000")))
    }
}
impl ParamR {
    pub fn summary(&self) -> String {
        match self {
            ParamR::Raw(s) => s.raw.clone(),
        }
    }
}

/// resistor device class
#[derive(Debug, Clone)]
pub struct R {
    /// parameters of the resistor
    pub params: ParamR,
    /// graphic representation of the resistor
    pub graphics: &'static Graphics,
}
impl R {
    pub fn new() -> R {
        R {
            params: ParamR::default(),
            graphics: &DEFAULT_GRAPHICS,
        }
    }
}
