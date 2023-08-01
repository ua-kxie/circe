//! device definition for diodes (DXXXX)
// . model DMOD D

// DXXXXXXX n + n - mname < area = val > <m = val > < pj = val > <off >
// + < ic = vd > < temp = val > < dtemp = val >
// + < lm = val > < wm = val > < lp = val > < wp = val >

use super::super::params;
use super::Graphics;
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "D";

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