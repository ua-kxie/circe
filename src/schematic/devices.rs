// ex: Vgnd0 net1 0 0
// device Id, net at port, ground net '0', device voltage 0
mod devicetype;
mod deviceinstance;

use std::{rc::Rc, cell::RefCell, hash::Hasher};
use euclid::{Vector2D, Transform2D};
use iced::widget::canvas::Frame;
use std::hash::Hash;

use crate::{
    schematic::nets::{Drawable},
    transforms::{
        SSPoint, VSBox, VCTransform, SchematicSpace, SSBox, VSPoint
    }, 
};

use self::devicetype::Port;

// pub use self::deviceinstance::DeviceInstance;
// use self::devicetype::DeviceType;

trait SpiceDevice {
    fn SpiceLine(&self) -> String;
}

struct Interactable {
    bounds: SSBox,
    tentative: bool,
    selected: bool,
}

impl Interactable {
    fn new() -> Self {
        Interactable { bounds: SSBox::default(), tentative: false, selected: false }
    }
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
    pub fn new_with_ord(ord: usize) -> Self {
        Identifier { id_prefix: &self::PREFIX_R, id: ord, custom: None }
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
    pts: Vec<Vec<VSPoint>>,
    ports: Vec<devicetype::Port>,
    marker: core::marker::PhantomData<T>,
}
impl<T> Graphics<T> {
    fn default_r() -> Self {
        Graphics { 
            pts: vec![
                vec![
                    VSPoint::new(0., 3.),
                    VSPoint::new(0., -3.),
                ],
                vec![
                    VSPoint::new(-1., 2.),
                    VSPoint::new(-1., -2.),
                    VSPoint::new(1., -2.),
                    VSPoint::new(1., 2.),
                    VSPoint::new(-1., 2.),
                ],
            ],
            ports: vec![
                Port {name: "+", offset: SSPoint::new(0, 3)},
                Port {name: "-", offset: SSPoint::new(0, -3)},
            ], 
            marker: core::marker::PhantomData 
        }
    }
    fn default_gnd() -> Self {
        Graphics { 
            pts: vec![
                vec![
                    VSPoint::new(0., 2.),
                    VSPoint::new(0., -1.)
                ],
                vec![
                    VSPoint::new(0., -2.),
                    VSPoint::new(1., -1.),
                    VSPoint::new(-1., -1.),
                    VSPoint::new(0., -2.),
                ],
            ],
            ports: vec![
                Port {name: "gnd", offset: SSPoint::new(0, 2)}
            ], 
            marker: core::marker::PhantomData 
        }
    }
}
trait DeviceType <T> {
    fn default_graphics() -> Graphics<T>;
}
struct R;
impl <T> DeviceType<T> for R {
    fn default_graphics() -> Graphics<T> {
        Graphics::default_r()
    }
}
struct Gnd;
impl <T> DeviceType<T> for Gnd {
    fn default_graphics() -> Graphics<T> {
        Graphics::default_gnd()
    }
}

struct SingleValue <T> {
    value: f32,
    marker: core::marker::PhantomData<T>,
}
impl <T> SingleValue<T> {
    fn new() -> Self {
        SingleValue { value: 0.0, marker: core::marker::PhantomData }
    }
}
enum Param <T> {
    Value(SingleValue<T>),
}
struct Device <T> {
    id: Identifier,
    interactable: Interactable,
    transform: Transform2D<i16, SchematicSpace, SchematicSpace>,
    graphics: Rc<Graphics<T>>,  // contains ports, bounds - can be edited, but contents of GraphicsR cannot be edited (from schematic editor)
    params: Param<T>,
}
impl <T> Device<T> {
    pub fn new_with_ord(ord: usize, graphics: Rc<Graphics<T>>) -> Self {
        Device { 
            id: Identifier::new_with_ord(ord), 
            interactable: Interactable::new(), 
            transform: Transform2D::identity(), 
            graphics, 
            params: Param::Value(SingleValue::<T>::new())
        }
    }
}
struct DeviceSet <T> where T: DeviceType<T> {
    vec: Vec<Rc<RefCell<Device<T>>>>, 
    wm: usize,
    graphics: Vec<Rc<Graphics<T>>>,
}
impl<T> DeviceSet<T> where T: DeviceType<T> {
    fn new_instance(&mut self) -> Rc<RefCell<Device<T>>> {
        self.wm += 1;
        let t = Rc::new(RefCell::new(Device::<T>::new_with_ord(self.wm, self.graphics[0])));
        self.vec.push(t.clone());
        t
    }
    fn new() -> Self {
        DeviceSet { vec: vec![], wm: 0, graphics: vec![Rc::new(T::default_graphics())] }
    }
}

pub struct Devices {
    set_r: DeviceSet<R>,
    set_gnd: DeviceSet<Gnd>,
}

impl Default for Devices {
    fn default() -> Self {
        Devices::new()
    }
}

impl Drawable for Devices {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.devices_vec {
            d.borrow().draw_persistent(vct, vcscale, frame);
        }
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.devices_vec {
            if d.borrow().selected {
                d.borrow().draw_selected(vct, vcscale, frame);
            }
        }
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in self.devices_vec.iter().filter(|&d| d.borrow().tentative) {
            d.borrow().draw_preview(vct, vcscale, frame);
        }
    }
}

impl Devices {
    pub fn ports_ssp(&self) -> Vec<SSPoint> {
        self.devices_vec.iter().flat_map(|d| d.borrow().ports_ssp()).collect()
    }
    pub fn tentatives_to_selected(&mut self) {
        for d in self.devices_vec.iter().filter(|&d| d.borrow().tentative) {
            d.borrow_mut().selected = true;
            d.borrow_mut().tentative = false;
        }
    }
    pub fn move_selected(&mut self, ssv: Vector2D<i16, SchematicSpace>) {
        for d in self.devices_vec.iter().filter(|&d| d.borrow().selected) {
            d.borrow_mut().pre_translate(ssv.cast_unit());
            d.borrow_mut().selected = false;
        }
    }
    pub fn draw_selected_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in self.devices_vec.iter().filter(|&d| d.borrow().selected) {
            d.borrow().draw_preview(vct, vcscale, frame);
        }
    }
    pub fn clear_selected(&mut self) {
        for d in &self.devices_vec {
            d.borrow_mut().selected = false;
        }
    }
    pub fn clear_tentatives(&mut self) {
        for d in &self.devices_vec {
            d.borrow_mut().tentative = false;
        }
    }
    pub fn bounding_box(&self) -> VSBox {
        let pts = self.devices_vec.iter().flat_map(|d| [d.borrow().bounds().min, d.borrow().bounds().max].into_iter());
        SSBox::from_points(pts).cast().cast_unit()
    }
    pub fn push(&mut self, di: DeviceInstance) {
        self.devices_vec.push(Rc::new(di.into()));
    }
    pub fn iter(&self) -> std::slice::Iter<Rc<RefCell<DeviceInstance>>> {
        self.devices_vec.iter()
    }
    pub fn place_res(&mut self, ssp: SSPoint) -> DeviceInstance {
        DeviceInstance::new_res(ssp, self.res.clone())
    }
    pub fn delete_selected(&mut self) {
        self.devices_vec = self.devices_vec.iter().filter_map(|e| {
            if !e.borrow().selected {Some(e.clone())} else {None}
        }).collect()
    }
    fn new() -> Self {
        Devices { devices_vec: vec![], res: Rc::new(DeviceType::new_res()) }
    }
    pub fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        for d in &self.devices_vec {
            if d.borrow().ports_occupy_ssp(ssp) {
                return true;
            }
        }
        return false;
    }
}


