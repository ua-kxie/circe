//! device definition for independent current source (IXXXX)

// IYYYYYYY N + N - <<DC > DC / TRAN VALUE > < AC < ACMAG < ACPHASE > > >
// + < DISTOF1 < F1MAG < F1PHASE > > > < DISTOF2 < F2MAG < F2PHASE > > >

use crate::schematic::devices::port::Port;
use crate::schematic::devices::strokes::CirArc;
use crate::schematic::interactable::Interactable;
use crate::transforms::{SSBox, SSPoint, VSPoint};

use super::super::params;
use super::Graphics;
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "I";

lazy_static! {
    static ref DEFAULT_GRAPHICS: Graphics = Graphics {
        pts: vec![
            vec![VSPoint::new(-0.50, -0.25), VSPoint::new(0.50, -0.25),],
            vec![VSPoint::new(0.00, -1.00), VSPoint::new(0.00, -3.00),],
            vec![VSPoint::new(0.00, 3.00), VSPoint::new(0.00, 1.00),],
            vec![VSPoint::new(0.00, -0.75), VSPoint::new(-0.50, -0.25),],
            vec![VSPoint::new(0.50, -0.25), VSPoint::new(0.00, -0.75),],
            vec![VSPoint::new(0.00, -0.25), VSPoint::new(0.00, 0.75),],
        ],
        cirarcs: vec![CirArc::from_triplet(
            VSPoint::new(0.00, 0.00),
            VSPoint::new(0.00, -1.00),
            VSPoint::new(0.00, -1.00)
        ),],
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

#[derive(Debug, Clone)]
pub enum ParamI {
    Raw(params::Raw),
}
impl Default for ParamI {
    fn default() -> Self {
        ParamI::Raw(params::Raw::new(String::from("1u")))
    }
}
impl ParamI {
    pub fn summary(&self) -> String {
        match self {
            ParamI::Raw(s) => s.raw.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct I {
    pub params: ParamI,
    pub graphics: &'static Graphics,
}
impl I {
    pub fn new() -> I {
        I {
            params: ParamI::default(),
            graphics: &DEFAULT_GRAPHICS,
        }
    }
}
