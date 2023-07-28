//! device definition for bjt (QXXXX)
// .model QMOD1 PNP
// .model QMOD3 NPN level=4
// port order: c b e (analogous to mosfet d g s), optional body node, junction temperature
// followed by model name

// QXXXXXXX nc nb ne <ns > <tj > mname < area = val > < areac = val >
// + < areab = val > <m= val > <off > < ic = vbe , vce > < temp = val >
// + < dtemp = val >

// need a way to define models once in the netlist

use super::super::params;
use super::Graphics;
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "Q";

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
