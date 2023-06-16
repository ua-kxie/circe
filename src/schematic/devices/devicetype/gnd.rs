use crate::transforms::{SSPoint, VSPoint, SSBox};
use super::{Graphics, Port};
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "VGND";

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
        circles: vec![],
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
impl ParamGnd {
    pub fn summary(&self) -> String {
        String::from("0 0")
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