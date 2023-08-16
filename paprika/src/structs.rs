#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, PartialOrd, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// Struct known as vecvaluesall in Ngspice User's Manual
pub struct PkVecvaluesall {
    pub count: i32,
    pub index: i32,
    pub vecsa: Vec<Box<PkVecvalues>>,
}
#[derive(Clone, PartialEq, PartialOrd, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// Struct known as vecvalues in Ngspice User's Manual
pub struct PkVecvalues {
    pub name: String,
    pub creal: f64,
    pub cimag: f64,
    pub is_scale: bool,
    pub is_complex: bool,
}

impl From<PkVecvalues> for num::Complex<f32> {
    fn from(value: PkVecvalues) -> Self {
        Self { re: value.creal as f32, im: value.cimag as f32 }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
/// Struct known as vecinfoall in Ngspice User's Manual
pub struct PkVecinfoall {
    pub name: String,
    pub title: String,
    pub date: String,
    pub stype: String,
    pub count: i32,
    pub vecs: Vec<Box<PkVecinfo>>,
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[allow(dead_code)]
/// Struct known as vecinfo in Ngspice User's Manual
pub struct PkVecinfo {
    pub number: i32,
    pub name: String,
    pub is_real: bool,
    pub pdvec: usize,
    pub pdvecscale: usize,
}
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PkComplex {
    pub real: f64,
    pub imag: f64,
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct PkVectorinfo {
    pub name: String,
    pub stype: i32,
    pub flag: i16,
    pub realdata: Option<Vec<f64>>,
    pub compdata: Option<Vec<num::Complex<f64>>>,
    pub length: i32,
}
