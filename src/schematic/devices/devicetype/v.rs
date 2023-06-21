use crate::transforms::{SSPoint, VSPoint, SSBox};
use super::super::params;
use super::{Graphics, Port};
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "V";

lazy_static! {
    static ref DEFAULT_GRAPHICS: Graphics = Graphics { 
        pts: vec![
            vec![
                VSPoint::new(0., 3.),
                VSPoint::new(0., 1.5)
            ],
            vec![
                VSPoint::new(0., -1.5),
                VSPoint::new(0., -3.),
            ],
            vec![
                VSPoint::new(-0.5, -1.),
                VSPoint::new(0.5, -1.),
            ],
            vec![
                VSPoint::new(0., 1.5),
                VSPoint::new(0., 0.5),
            ],
            vec![
                VSPoint::new(-0.5, 1.0),
                VSPoint::new(0.5, 1.0),
            ],
        ],
        circles: vec![
            (VSPoint::origin(), 1.5),
        ],
        ports: vec![
            Port {name: "+", offset: SSPoint::new(0, 3)},
            Port {name: "-", offset: SSPoint::new(0, -3)},
        ], 
        bounds: SSBox::new(SSPoint::new(-2, 3), SSPoint::new(2, -3)), 
    };
}

#[derive(Debug)]
pub enum ParamV  {
    Raw(params::Raw),
}
impl Default for ParamV {
    fn default() -> Self {
        ParamV::Raw(params::Raw::new(String::from("3.3")))
    }
}
impl ParamV {
    pub fn summary(&self) -> String {
        match self {
            ParamV::Raw(s) => {
                s.raw.clone()
            },
        }
    }
}

#[derive(Debug)]
pub struct V {
    pub params: ParamV,
    pub graphics: &'static Graphics,
}
impl V {
    pub fn new() -> V {
        V {params: ParamV::default(), graphics: &DEFAULT_GRAPHICS}
    }
}