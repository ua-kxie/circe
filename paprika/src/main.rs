use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

// use ::paprika;
use colored::Colorize;
use paprika::*;
#[allow(dead_code)]
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

fn main() {
    let mut spice = PkSpice::<Manager>::new(std::ffi::OsStr::new("ngspice.dll")).unwrap();
    let buf = Arc::new(RwLock::new(VecDeque::<String>::with_capacity(10)));
    let manager = Arc::new(Manager::new(buf));

    spice.init(Some(manager)); // register
    spice.command("source tran.cir"); // results pointer array starts at same address
    spice.command("tran 10u 10m"); // ngspice recommends sending in control statements separately, not as part of netlist

    spice.init(None); // unregister
    spice.command("echo echo command");

    // spice.command("source ac.cir");  // results pointer array starts at same address
    // spice.command("ac dec 10 1 100k");  // ngspice recommends sending in control statements separately, not as part of netlist

    // // dbg!(manager.lib.running());
    // let a = manager.lib.get_vec_info("tran1.time");
    // let a1 = a.realdata.unwrap();
    // let b = manager.lib.get_vec_info("ac1.frequency");
    // let b1 = b.compdata.unwrap();

    // dbg!(manager.lib.get_allplots());
    // dbg!(manager.lib.get_curplotname());
    // dbg!(manager.lib.get_allvecs("ac2"));
    // dbg!(manager.lib.get_vec_info("frequency"));
    // let a = String::from("echo hello");
    // manager.lib.command(&a.as_str());

    let mut line = String::new();
    loop {
        line.clear();
        let _ = std::io::stdin().read_line(&mut line).unwrap();
        match line.as_str().split_once("\r\n") {
            Some(tup) => {
                spice.command(tup.0);
            }
            None => {
                spice.command(line.as_str());
            } // this should only happen for blank inputs {println!("{:?}", line);},
        }
    }
}
