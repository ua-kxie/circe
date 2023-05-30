use core::fmt;
use std::{rc::Rc, collections::HashSet, hash::{Hash, Hasher}, cell::RefCell};

// device
//  type
// R L C M V I X
// subcircuit type need to support different graphics
/*
device instance
    device type enum
        parameter set
        graphics
        usize id
    id/name

devices
    hashset R - hash usize id
    hashset V
    hashset Vgnd
    hashset X

R device instance
    R parameter set enum
    R graphics - free to choose as long as ports count match - includes bounds
    transform
    transformed bounds
    tentative
    selected

V device instance
    V parameter set enum
    V graphics - free to choose as long as ports count match - includes bounds
    transform
    transformed bounds
    tentative
    selected

 */
//
trait SpiceDevice {
    fn SpiceLine(&self) -> String {
        String::from("placeholder")
    }
}

enum ParamR {
    Value(f32),
}

struct Interactable {
    bounds: usize,
    tentative: bool,
    selected: bool,
}

impl Interactable {
    fn curpos_moved_ssp(&self, curpos_ssp: usize) {

    }
}
struct Identifier {
    id_prefix: &'static [char; 1],  // prefix which determines device type in NgSpice
    id: usize,  // this can be whatever - changed to whatever whenever and used to generate Id if custom is None
    custom: Option<String>,  // if some, is set by the user - must use this as is for id
}
impl Identifier {
    pub fn ng_id(&self) -> String {
        let mut ret = String::new();
        for c in self.id_prefix {
            ret.push(*c);
        }
        if let Some(s) = self.custom {
            ret.push_str(&s);
        } else {
            ret.push_str(&format!("{}", self.id));
        }
        ret
    }
}
const R: [char; 1] = ['R'];
struct DeviceR {
    id: Identifier,
    graphics: Rc<GraphicsR>,  // contains ports, bounds - can be edited, but contents of GraphicsR cannot be edited (from schematic editor)
    params: ParamR,
    transform: usize,
    interactable: Interactable,
}

trait DeviceType {
    type T: DeviceType + Eq + Hash;

    fn new(id: usize) -> Self::T;
}
struct DeviceSet <T> where T: Hash + Eq + DeviceType {
    set: HashSet<Rc<T>>, 
    wm: usize,
}
impl<T> DeviceSet<T> where T: Hash + Eq + DeviceType<T=T> {
    fn new_instance(&mut self) -> Rc<T> {
        self.wm += 1;
        let t = Rc::new(T::new(self.wm));
        self.set.insert(t);
        t
    }
}
// refcell doesnt impl Hash due to mutability...
// - make Rc<DeviceInstance{id: usize, RefCell<other stuff>}>
// create new and delete if id is changed

/*
- or -
Rc<RefCell<DeviceInstance>>

struct DeviceSet<DeviceR> {
    devices: Vec<DeviceR>,
    wm: usize,
}

and just throw error on schematic check if multiple devices occupy same id
*/
fn main() {
    
}