use crate::transforms::{SSPoint, VSPoint, SSBox};
use super::{Graphics, Port};
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "R";

lazy_static! {
    static ref DEFAULT_GRAPHICS: Graphics = Graphics { 
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
        ],
        ports: vec![
            Port {name: "+", offset: SSPoint::new(0, 3)},
            Port {name: "-", offset: SSPoint::new(0, -3)},
        ], 
        bounds: SSBox::new(SSPoint::new(-2, 3), SSPoint::new(2, -3)), 
    };
}

#[derive(Debug)]
pub struct Raw  {
    raw: String,
}
impl Raw {
    fn new(raw: String) -> Self {
        Raw { raw }
    }
    pub fn set(&mut self, new: String) {
        self.raw = new;
    }
}

#[derive(Debug)]
pub struct SingleValue  {
    value: f32,
}
impl SingleValue {
    fn new(value: f32) -> Self {
        SingleValue { value }
    }
}


#[derive(Debug)]
pub enum ParamR  {
    Raw(Raw),
    Value(SingleValue),
}
impl Default for ParamR {
    fn default() -> Self {
        ParamR::Raw(Raw::new(String::from("1000")))
    }
}
impl ParamR {
    pub fn summary(&self) -> String {
        match self {
            ParamR::Value(v) => {
                std::format!("{}", v.value)
            },
            ParamR::Raw(s) => {
                s.raw.clone()
            },
        }
    }
}

#[derive(Debug)]
pub struct R {
    pub params: ParamR,
    pub graphics: &'static Graphics,
}
impl R {
    pub fn new() -> R {
        R {params: ParamR::default(), graphics: &DEFAULT_GRAPHICS}
    }
}