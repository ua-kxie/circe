use std::collections::VecDeque;
use std::ffi::OsStr;
use std::sync::{Arc, RwLock};

use paprika::*;

struct Manager {
    sharedres: Arc<RwLock<VecDeque<String>>>,
    quit_flag: bool,
    vec_char: Vec<String>,
    vec_stat: Vec<String>,
    vec_pkvecinfoall: Vec<PkVecinfoall>,
    vec_pkvecvalsall: Vec<PkVecvaluesall>,
}
impl Manager {
    fn new(arvs: Arc<RwLock<VecDeque<String>>>) -> Manager {
        Manager {
            sharedres: arvs,
            quit_flag: false,
            vec_char: Vec::<String>::new(),
            vec_stat: Vec::<String>::new(),
            vec_pkvecinfoall: Vec::<PkVecinfoall>::new(),
            vec_pkvecvalsall: Vec::<PkVecvaluesall>::new(),
        }
    }
}
#[allow(unused_variables)]
impl paprika::PkSpiceManager for Manager {
    fn cb_send_char(&mut self, msg: String, id: i32) {
        let mut arvs = self.sharedres.write().unwrap();
        (*arvs).push_back(msg.clone());
    }
    fn cb_send_stat(&mut self, msg: String, id: i32) {}
    fn cb_ctrldexit(&mut self, status: i32, is_immediate: bool, is_quit: bool, id: i32) {
        println!(
            "ctrldexit {}; {}; {}; {};",
            status, is_immediate, is_quit, id
        );
        self.quit_flag = true;
    }
    fn cb_send_init(&mut self, pkvecinfoall: PkVecinfoall, id: i32) {
        self.vec_pkvecinfoall.push(pkvecinfoall);
    }
    fn cb_send_data(&mut self, pkvecvaluesall: PkVecvaluesall, count: i32, id: i32) {
        self.vec_pkvecvalsall.push(pkvecvaluesall);
    }
    fn cb_bgt_state(&mut self, is_fin: bool, id: i32) {
        println!("bgt_state {}; {};", is_fin, id);
    }
}

#[test]
fn test_cmd_echo() {
    let mut spice = PkSpice::<Manager>::new(OsStr::new("ngspice.dll")).unwrap();
    let buf = Arc::new(RwLock::new(VecDeque::<String>::with_capacity(10)));
    let manager = Arc::new(Manager::new(buf.clone()));

    spice.init(Some(manager)); // register

    spice.command("echo echo command");
    let s = (*buf.write().unwrap()).pop_back().unwrap();
    assert_eq!(s, "stdout echo command");
    spice.command("quit");
} // cannot run tests in parallel

#[test]
fn test_dcop() {
    let mut spice = PkSpice::<Manager>::new(OsStr::new("ngspice.dll")).unwrap();
    let buf = Arc::new(RwLock::new(VecDeque::<String>::with_capacity(10)));
    let manager = Arc::new(Manager::new(buf.clone()));

    spice.init(Some(manager)); // register

    spice.command("source dcop.cir");
    spice.command("op");
    spice.command("quit");
}
