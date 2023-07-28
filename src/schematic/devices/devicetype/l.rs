//! device definition for inductors (LXXXX)

// LYYYYYYY n + n - < value > < mname > < nt = val > <m = val >
// + < scale = val > < temp = val > < dtemp = val > < tc1 = val >
// + < tc2 = val > < ic = init_condition >

use super::super::params;
use super::Graphics;
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "L";

lazy_static! {
    static ref DEFAULT_GRAPHICS: Graphics = todo!();
}

#[derive(Debug, Clone)]
pub enum ParamM {
    Raw(params::Raw),
}
impl Default for ParamV {
    fn default() -> Self {
        ParamV::Raw(params::Raw::new(String::from("")))
    }
}
impl ParamV {
    pub fn summary(&self) -> String {
        match self {
            ParamV::Raw(s) => s.raw.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct V {
    pub params: ParamV,
    pub graphics: &'static Graphics,
}
impl V {
    pub fn new() -> V {
        V {
            params: ParamV::default(),
            graphics: &DEFAULT_GRAPHICS,
        }
    }
}
