#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use colored::Colorize;
use libc::c_void;
use paprika::{PkVecinfoall, PkVecvaluesall, PkSpice, self};

#[allow(dead_code)]
struct Manager {
}
impl Manager {
    fn new() -> Manager {
        Manager {
        }
    }
}
#[allow(unused_variables)]
impl paprika::PkSpiceManager for Manager {
    fn cb_send_char(&mut self, msg: String, id: i32) {
        let opt = msg.split_once(' ');
        let (token, msgs) = match opt {
            Some(tup) => (tup.0, tup.1),
            None => (msg.as_str(), msg.as_str()),
        };
        let msgc = match token {
            "stdout" => msgs.green(),
            "stderr" => msgs.red(),
            _ => msg.magenta().strikethrough(),
        };
        println!("{}", msgc);
    }
    fn cb_send_stat(&mut self, msg: String, id: i32) {
        println!("{}", msg.blue());
    }
    fn cb_ctrldexit(&mut self, status: i32, is_immediate: bool, is_quit: bool, id: i32) {
        println!(
            "ctrldexit {}; {}; {}; {};",
            status, is_immediate, is_quit, id
        );
    }
    fn cb_send_init(&mut self, pkvecinfoall: PkVecinfoall, id: i32) {
        
    }
    fn cb_send_data(&mut self, pkvecvaluesall: PkVecvaluesall, count: i32, id: i32) {
        
    }
    fn cb_bgt_state(&mut self, is_fin: bool, id: i32) {
        println!("bgt_state {}; {};", is_fin, id);
    }
}

fn command(cmdstr: &str) -> bool {
    unsafe {
        let ret = if cmdstr.is_empty() {
            ngSpice_Command(core::ptr::null_mut())
        }
        // have users spawn their own threads instead
        else if cmdstr.find("bg_") == Some(0) {
            0
        } else {
            let ccmdstr = std::ffi::CString::new(cmdstr).unwrap().into_raw();
            let ret0 = ngSpice_Command(ccmdstr);
            let _ = std::ffi::CString::from_raw(ccmdstr);
            ret0
        };
        ret != 0
    }
}

fn startup() {
    unsafe {
        let manager = Arc::new(Manager::new());
        ngSpice_Init(
            Some(paprika::cbw_send_char::<Manager>), 
            Some(paprika::cbw_send_stat::<Manager>), 
            Some(paprika::cbw_controlled_exit::<Manager>), 
            None, 
            None,
            Some(paprika::cbw_bgthread_running::<Manager>),
            &*manager as *const _ as *mut c_void,
        );
        command("echo echo command");
    }
}

fn main() {
    startup()
}
