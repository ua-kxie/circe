//! General Notes
//!
//! Thread Safety
//!
//! Analysis commands like `tran 10u 10m` in the netlist is executed immediately. Same effect as sending `tran 10u 10m` through `NgSpice_Command`
//! after loading the netlist.
//! Dot analysis commands like `.tran 10u 10m` in the netlist is executed after `run` or `bg_run` is sent through `NgSpice_Command`.
//! Safety must assume that callbacks are called from parallel thread after commanding `bg_run`.

use std::{
    ffi::{OsStr, OsString},
    sync::Arc,
};

use libc::*;
#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;
use libloading::Library;
mod structs;
pub use structs::*;
mod ngspice;
use ngspice::*;

#[derive(Debug)]
pub enum PkSpiceError {
    /// Sharedspice library is not found
    SharedspiceNotFound(OsString),
}

type NgSpiceInit = extern "C" fn(
    Option<unsafe extern "C" fn(*const c_char, c_int, *const c_void) -> c_int>,
    Option<unsafe extern "C" fn(*const c_char, c_int, *const c_void) -> c_int>,
    Option<unsafe extern "C" fn(c_int, bool, bool, c_int, *const c_void) -> c_int>,
    Option<
        unsafe extern "C" fn(*const ngspice::NgVecvaluesall, c_int, c_int, *const c_void) -> c_int,
    >,
    Option<unsafe extern "C" fn(*const ngspice::NgVecinfoall, c_int, *const c_void) -> c_int>,
    Option<unsafe extern "C" fn(bool, c_int, *const c_void) -> c_int>,
    *const c_void,
) -> c_int;
type NgSpiceCommand = extern "C" fn(*const c_char) -> c_int;
type NgSpiceVecInfo = extern "C" fn(*const c_char) -> *const NgVectorinfo;
type NgSpiceCurPlot = extern "C" fn() -> *const c_char;
type NgSpiceAllPlots = extern "C" fn() -> *const *const c_char;
type NgSpiceAllVecs = extern "C" fn(*const c_char) -> *const *const c_char;
type NgSpiceRunning = extern "C" fn() -> bool;

#[allow(dead_code)]
struct VTableV0 {
    init: RawSymbol<NgSpiceInit>,

    command: RawSymbol<NgSpiceCommand>,
    get_vec_info: RawSymbol<NgSpiceVecInfo>,

    get_cur_plot: RawSymbol<NgSpiceCurPlot>,
    get_all_plots: RawSymbol<NgSpiceAllPlots>,
    get_all_vecs: RawSymbol<NgSpiceAllVecs>,
    is_running: RawSymbol<NgSpiceRunning>,
}

impl VTableV0 {
    unsafe fn get_symbol<T>(lib: &Library, sname: &[u8]) -> RawSymbol<T> {
        let symbol = lib.get(sname).unwrap();
        libloading::Symbol::<T>::into_raw(symbol)
    }

    unsafe fn new(lib: &Library) -> VTableV0 {
        // get symbols (same order as they appear in sharedspice.h)
        VTableV0 {
            init: VTableV0::get_symbol::<NgSpiceInit>(lib, b"ngSpice_Init\0"),
            // b"ngSpice_Init_Sync\0";
            command: VTableV0::get_symbol::<NgSpiceCommand>(lib, b"ngSpice_Command\0"),
            get_vec_info: VTableV0::get_symbol::<NgSpiceVecInfo>(lib, b"ngGet_Vec_Info\0"),
            // b"ngCM_Input_Path\0";
            // b"ngGet_Evt_NodeInfo\0";
            // b"ngSpice_AllEvtNodes\0";
            // b"ngSpice_Init_Evt\0";
            // b"ngSpice_Circ\0";
            get_cur_plot: VTableV0::get_symbol::<NgSpiceCurPlot>(lib, b"ngSpice_CurPlot\0"),
            get_all_plots: VTableV0::get_symbol::<NgSpiceAllPlots>(lib, b"ngSpice_AllPlots\0"),
            get_all_vecs: VTableV0::get_symbol::<NgSpiceAllVecs>(lib, b"ngSpice_AllVecs\0"),
            is_running: VTableV0::get_symbol::<NgSpiceRunning>(lib, b"ngSpice_running\0"),
            // b"ngSpice_SetBkpt\0";
        }
    }
}

pub trait PkSpiceManager {
    /// Callback known as SendChar in Ngspice User's Manual
    fn cb_send_char(&mut self, msg: String, id: i32);
    /// Callback known as SendStat in Ngspice User's Manual
    fn cb_send_stat(&mut self, msg: String, id: i32);
    /// Callback known as ControlledExit in Ngspice User's Manual
    fn cb_ctrldexit(&mut self, status: i32, is_immediate: bool, is_quit: bool, id: i32);
    /// Callback known as SendData in Ngspice User's Manual
    fn cb_send_data(&mut self, pkvecvaluesall: PkVecvaluesall, count: i32, id: i32);
    /// Callback known as SendInitData in Ngspice User's Manual
    fn cb_send_init(&mut self, pkvecinfoall: PkVecinfoall, id: i32);
    /// Callback known as BGThreadRunning in Ngspice User's Manual
    fn cb_bgt_state(&mut self, is_fin: bool, id: i32);
}
/// Represents a link to the sharedspice library
pub struct PkSpice<T>
where
    T: PkSpiceManager,
{
    #[allow(dead_code)]
    library: Library,
    api: VTableV0,
    manager: Option<Arc<T>>,
}

impl<T> PkSpice<T>
where
    T: PkSpiceManager,
{
    /// Links to a sharedspice library given by path.
    /// Returns error if the file given by path does not exist.
    /// Crashes if any expected symbols are not found, which will happen if path points to an incorrect file, or to a much older version of sharedspice.
    pub fn new(path: &std::ffi::OsStr) -> Result<PkSpice<T>, PkSpiceError> {
        unsafe {
            let lib = match Library::new(path) {
                Ok(lib) => lib,
                Err(_) => {
                    return Err(PkSpiceError::SharedspiceNotFound(path.to_os_string()));
                }
            };
            let vtable = VTableV0::new(&lib); // todo handle symbol not found
            Ok(PkSpice {
                library: lib,
                api: vtable,
                manager: None,
            })
        }
    }
    /// API function known as ngSpice_Init in Ngspice User's Manual
    pub fn init(&mut self, manager: Option<Arc<T>>) -> i32 {
        // drop existing manager
        // keep reference to new manager
        unsafe {
            match manager {
                Some(m) => {
                    let ret1 = (self.api.init)(
                        Some(cbw_send_char::<T>),
                        Some(cbw_send_stat::<T>),
                        Some(cbw_controlled_exit::<T>),
                        Some(cbw_send_data::<T>),
                        Some(cbw_send_init_data::<T>),
                        Some(cbw_bgthread_running::<T>),
                        &*m as *const _ as *const c_void,
                    );
                    self.manager = Some(m); // drop the previous manager, AFTER the new manager is registered
                    ret1
                }
                None => {
                    let ret1 =
                        (self.api.init)(None, None, None, None, None, None, std::ptr::null());
                    self.manager = None; // drop the previous manager, AFTER the new manager is registered
                    ret1
                }
            }
        }
    }
    /// API function known as ngSpice_Command in Ngspice User's Manual
    /// If cmdstr is an empty string, NULL is sent to ngSpice_Command, which clears the internal control structures.
    pub fn command(&self, cmdstr: &str) -> bool {
        unsafe {
            let ret = if cmdstr.is_empty() {
                (self.api.command)(std::ptr::null())
            }
            // have users spawn their own threads instead
            else if cmdstr.find("bg_") == Some(0) {
                0
            } else {
                let ccmdstr = std::ffi::CString::new(cmdstr).unwrap();
                (self.api.command)(ccmdstr.as_ptr())
            };
            ret != 0
        }
    }

    pub fn get_vec_info(&self, vecname: &str) -> PkVectorinfo {
        unsafe {
            let cvecname = std::ffi::CString::new(vecname).unwrap();
            let pvectorinfo = (self.api.get_vec_info)(cvecname.as_ptr());
            (*pvectorinfo).to_pk()
        }
    }

    pub fn get_cur_plot(&self) -> String {
        unsafe {
            let pcstr = (self.api.get_cur_plot)();
            std::ffi::CStr::from_ptr(pcstr)
                .to_str()
                .unwrap()
                .to_string()
        }
    }

    pub fn get_all_plots(&self) -> Vec<String> {
        unsafe {
            let ppcstr = (self.api.get_all_plots)();
            c_strings(ppcstr)
        }
    }

    pub fn get_all_vecs(&self, plotname: &str) -> Vec<String> {
        unsafe {
            let cplotname = std::ffi::CString::new(plotname).unwrap();
            let ppcstr = (self.api.get_all_vecs)(cplotname.as_ptr());
            c_strings(ppcstr)
        }
    }

    pub fn is_running(&self) -> bool {
        unsafe { (self.api.is_running)() }
    }
}

unsafe fn c_strings(ptr: *const *const c_char) -> Vec<String> {
    // safety requires
    // all pointers point to valid memory
    // pointer to array of null-terminated array of pointers, each of which point to a null-terminated string
    let mut len = 0;
    loop {
        if (*(ptr.add(len))).is_null() {
            break;
        } else {
            len += 1
        }
    }
    let s = std::slice::from_raw_parts(ptr, len);
    let mut vec = Vec::<String>::with_capacity(len);
    for &srcs in s.iter() {
        vec.push(std::ffi::CStr::from_ptr(srcs).to_str().unwrap().to_owned());
    }
    vec
}
