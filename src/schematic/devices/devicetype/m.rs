//! device definition for mosfets (MXXXX)
// .model n1 nmos level=54 version=4.8.2
// .model p1 pmos level=54 version=4.8.2
// port order: d g s b
// followed by model name

// MXXXXXXX nd ng ns nb mname <m = val > <l = val > <w = val >
// + < ad = val > < as = val > < pd = val > < ps = val > < nrd = val >
// + < nrs = val > <off > < ic = vds , vgs , vbs > < temp =t >

// need a way to define models once in the netlist

use super::super::params;
use super::Graphics;
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "M";

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
