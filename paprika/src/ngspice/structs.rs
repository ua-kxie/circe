use crate::structs::*;
use libc::*;
use std::ffi::CStr;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct NgEvtData {
    dcop: c_double,
    step: c_int,
    node_value: *const c_char,
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct NgEvtSharedData {
    evt_dect: *const NgEvtData,
    num_steps: c_int,
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct NgComplex {
    cx_real: c_double,
    cx_imag: c_double,
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct NgVectorinfo {
    v_name: *const c_char,
    v_type: c_int,
    v_flag: c_short,
    v_realdata: *const c_double, // only one of real or complex is legit?
    v_compdata: *const NgComplex,
    v_length: c_int,
}
impl NgVectorinfo {
    pub unsafe fn to_pk(self) -> PkVectorinfo {
        let (real, comp) = match self.v_type {
            1 => {
                // real
                let cvec = std::slice::from_raw_parts(self.v_realdata, self.v_length as usize);
                // create vec containing 'count' number of PkVecvalues
                let mut vec = Vec::<f64>::with_capacity(self.v_length as usize);
                // for item in vecinfos_slice:
                for item in cvec.iter() {
                    // create native PkVecinfo and store into vec
                    vec.push(*item);
                }
                (Some(vec), None)
            } // real
            2 => {
                // complex
                let cvec = std::slice::from_raw_parts(self.v_compdata, self.v_length as usize);
                // create vec containing 'count' number of PkVecvalues
                let mut vec = Vec::<num::Complex<f64>>::with_capacity(self.v_length as usize);
                // for item in vecinfos_slice:
                for item in cvec.iter() {
                    // create native PkVecinfo and store into vec
                    vec.push(num::Complex::<f64> {
                        re: item.cx_real,
                        im: item.cx_imag,
                    });
                }
                (None, Some(vec))
            } // complex
            _ => (None, None), // dunno,
        };
        PkVectorinfo {
            name: std::ffi::CStr::from_ptr(self.v_name)
                .to_str()
                .unwrap()
                .to_string(),
            stype: self.v_type,
            flag: self.v_flag,
            realdata: real,
            compdata: comp,
            length: self.v_length,
        }
    }
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct NgVecinfo {
    number: c_int,
    vecname: *const c_char,
    is_real: bool,
    pdvec: *const c_void, // not elaborated in the docs - not sure if intended for use
    pdvecscale: *const c_void, // not elaborated in the docs - not sure if intended for use
}
impl NgVecinfo {
    pub unsafe fn to_pk(self) -> PkVecinfo {
        PkVecinfo {
            number: self.number,
            name: std::ffi::CStr::from_ptr(self.vecname)
                .to_str()
                .unwrap()
                .to_string(),
            is_real: self.is_real,
            pdvec: self.pdvec as usize,
            pdvecscale: self.pdvecscale as usize,
        }
    }
}
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct NgVecinfoall {
    name: *const c_char,
    title: *const c_char,
    date: *const c_char,
    type_: *const c_char,
    veccount: c_int,
    vecs: *const *const NgVecinfo,
}
impl NgVecinfoall {
    pub unsafe fn to_pk(self) -> PkVecinfoall {
        let vecinfos_slice = std::slice::from_raw_parts(self.vecs, self.veccount as usize);
        // create vec containing 'count' number of PkVecvalues
        let mut pkvecinfos = Vec::<Box<PkVecinfo>>::with_capacity(self.veccount as usize);
        // for item in vecinfos_slice:
        for item in vecinfos_slice.iter() {
            // create native PkVecinfo and store into vec
            pkvecinfos.push(Box::<PkVecinfo>::new((*(*item)).to_pk()));
        }
        // create native PkVecInfoall
        PkVecinfoall {
            name: CStr::from_ptr(self.name).to_str().unwrap().to_string(),
            title: CStr::from_ptr(self.title).to_str().unwrap().to_string(),
            date: CStr::from_ptr(self.date).to_str().unwrap().to_string(),
            stype: CStr::from_ptr(self.type_).to_str().unwrap().to_string(),
            count: self.veccount,
            vecs: pkvecinfos,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct NgVecvalues {
    name: *const c_char,
    creal: c_double,
    cimag: c_double,
    is_scale: bool,
    is_complex: bool,
}
impl NgVecvalues {
    pub unsafe fn to_pk(self) -> PkVecvalues {
        PkVecvalues {
            name: std::ffi::CStr::from_ptr(self.name)
                .to_owned()
                .into_string()
                .unwrap(),
            creal: self.creal,
            cimag: self.cimag,
            is_scale: self.is_scale,
            is_complex: self.is_complex,
        }
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct NgVecvaluesall {
    count: c_int,
    index: c_int,
    vecsa: *const *const NgVecvalues,
}
impl NgVecvaluesall {
    pub unsafe fn to_pk(self) -> PkVecvaluesall {
        let vecvals_slice = std::slice::from_raw_parts(self.vecsa, self.count as usize);
        // create vec containing 'count' number of PkVecvalues
        let mut pkvecvalues = Vec::<Box<PkVecvalues>>::with_capacity(self.count as usize);
        // for item in vecvals_slice:
        for item in vecvals_slice.iter() {
            // create native PkVecvalues and store into vec
            pkvecvalues.push(Box::<PkVecvalues>::new((*(*item)).to_pk()));
        }
        // create native PkVecvaluesall
        PkVecvaluesall {
            count: self.count,
            index: self.index,
            vecsa: pkvecvalues,
        }
    }
}
