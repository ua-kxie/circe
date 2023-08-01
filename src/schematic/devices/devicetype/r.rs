//! device definition for resistors (RXXXX)

use super::super::params;
use super::{Graphics, Port};
use crate::transforms::{SSBox, SSPoint, VSPoint};
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "R";

lazy_static! {
    static ref DEFAULT_GRAPHICS: Graphics = Graphics {
        pts: vec![
            vec![VSPoint::new(0., 3.), VSPoint::new(0., -3.),],
            vec![
                VSPoint::new(-1., 2.),
                VSPoint::new(-1., -2.),
                VSPoint::new(1., -2.),
                VSPoint::new(1., 2.),
                VSPoint::new(-1., 2.),
            ],
        ],
        circles: vec![],
        ports: vec![
            Port {
                name: "+".to_string(),
                offset: SSPoint::new(0, 3)
            },
            Port {
                name: "-".to_string(),
                offset: SSPoint::new(0, -3)
            },
        ],
        bounds: SSBox::new(SSPoint::new(-2, 3), SSPoint::new(2, -3)),
    };
}

/// Enumerates the different ways to specifify parameters for a resistor
#[derive(Debug, Clone)]
pub enum ParamR {
    /// specify the spice line directly (after id and port connections)
    Raw(params::Raw),
    /// specify the spice line by a single value
    Value(params::SingleValue),
}
impl Default for ParamR {
    fn default() -> Self {
        ParamR::Raw(params::Raw::new(String::from("1000")))
    }
}
impl ParamR {
    pub fn summary(&self) -> String {
        match self {
            ParamR::Value(v) => {
                std::format!("{}", v.value)
            }
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
