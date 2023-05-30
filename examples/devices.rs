use core::fmt;
use std::{rc::Rc, collections::HashSet, hash::{Hash, Hasher}, cell::RefCell};
use euclid::default::Transform2D;

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
    id_prefix: &'static [char],  // prefix which determines device type in NgSpice
    id: usize,  // avoid changing - otherwise, 
    custom: Option<String>,  // if some, is set by the user - must use this as is for id - if multiple instances have same, both should be highlighted
    // changing the id will break outputs which reference the old id. Otherwise it can be changed
    // 1. how to catch and highlight duplicates
    // 2. how to know id should not be changed (that it is referenced)
}
/*
duplicates:
    create hashset, for every identifier insert. if duplicate, save in second hashset
    every key in second hashset has duplicates
    iterate through devices and highlight every device with id which matches a key in second hashset

immutable identifier:
    abuse rwlock? references take read lock
    if mutation is desired, must acquire write lock - e.g. no read locks. 
 */
impl Identifier {
    pub fn ng_id(&self) -> String {
        let mut ret = String::new();
        for c in self.id_prefix {
            ret.push(*c);
        }
        if let Some(s) = &self.custom {
            ret.push_str(s);
        } else {
            ret.push_str(&format!("{}", self.id));
        }
        ret
    }
}
impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.ng_id().eq(&other.ng_id())
    }
}
impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ng_id().hash(state);
    }
}
const PREFIX_R: [char; 1] = ['R'];
struct Graphics <T> {
    // T is just an identifier so the graphic is not used for the wrong device type, analogous to ViewportSpace/SchematicSpace of euclid
    pts: Vec<Vec<euclid::default::Point2D<f32>>>,
    marker: core::marker::PhantomData<T>,
}

struct R;
struct SingleValue <T> {
    value: f32,
    marker: core::marker::PhantomData<T>,
}
enum Param <T> {
    Value(SingleValue<T>),
}
struct Device <T> {
    id: Identifier,
    interactable: Interactable,
    transform: Transform2D<f32>,
    graphics: Rc<Graphics<T>>,  // contains ports, bounds - can be edited, but contents of GraphicsR cannot be edited (from schematic editor)
    params: Param<T>,
}

trait DeviceType {
    type T: DeviceType;

    fn new(id: usize) -> Self::T;
}
struct DeviceSet <T> where T: DeviceType {
    vec: Vec<Rc<RefCell<T>>>, 
    wm: usize,
}
impl<T> DeviceSet<T> where T: DeviceType<T=T> {
    fn new_instance(&mut self) -> Rc<RefCell<T>> {
        self.wm += 1;
        let t = Rc::new(RefCell::new(T::new(self.wm)));
        self.vec.push(t.clone());
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