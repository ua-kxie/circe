use crate::transforms::{SSPoint, VSPoint, SSBox};
use super::{Graphics, Port};
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "GND";

lazy_static! {
    static ref DEFAULT_GRAPHICS: Graphics = Graphics { 
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
        ],
        ports: vec![
            Port {name: "gnd", offset: SSPoint::new(0, 2)}
        ], 
        bounds: SSBox::new(SSPoint::new(-1, 2), SSPoint::new(1, -2)), 
    };
}

#[derive(Debug)]
pub enum ParamGnd  {
    None,
}
impl Default for ParamGnd {
    fn default() -> Self {
        ParamGnd::None
    }
}

#[derive(Debug)]
pub struct Gnd {
    pub params: ParamGnd,
    pub graphics: &'static Graphics,
}
impl Gnd {
    pub fn new() -> Gnd {
        Gnd {params: ParamGnd::default(), graphics: &DEFAULT_GRAPHICS}
    }
}