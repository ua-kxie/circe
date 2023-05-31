// ex: Vgnd0 net1 0 0
// device Id, net at port, ground net '0', device voltage 0
mod devicetype;
mod deviceinstance;
use devicetype::Graphics;
use deviceinstance::{DeviceType, Device, R, Gnd};
pub use deviceinstance::DeviceExt;

use std::{rc::Rc, cell::RefCell, hash::Hasher, collections::HashSet};
use euclid::{Vector2D, Transform2D, Angle};
use iced::widget::canvas::Frame;

use crate::{
    schematic::nets::{Drawable},
    transforms::{
        SSPoint, VSBox, VCTransform, SchematicSpace, SSBox, VSPoint
    }, 
};

use by_address::ByAddress;

pub struct RcRDevice <T> (Rc<RefCell<Device<T>>>);

impl <T> PartialEq for RcRDevice<T> {
    fn eq(&self, other: &Self) -> bool {
        ByAddress(self.0.clone()) == ByAddress(other.0.clone())
    }
}

impl <T> std::hash::Hash for RcRDevice<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ByAddress(self.0.clone()).hash(state);
    }
}

struct DeviceSet <T> where T: DeviceType<T> {
    set: HashSet<RcRDevice<T>>, 
    wm: usize,
    graphics_resources: Vec<Rc<Graphics<T>>>,
}
impl<T> DeviceSet<T> where T: DeviceType<T> + 'static {
    fn new_instance(&mut self) -> Rc<RefCell<Device<T>>> {
        Rc::new(RefCell::new(Device::<T>::new_with_ord(self.wm, self.graphics_resources[0].clone())))
    }
    fn new() -> Self {
        DeviceSet { set: HashSet::new(), wm: 0, graphics_resources: vec![Rc::new(T::default_graphics())] }
    }
    fn devices_traits(&self) -> Vec<Rc<RefCell<dyn DeviceExt>>> {
        self.set.iter().map(|x| x.0.clone() as Rc<RefCell<dyn DeviceExt>>).collect()
    }
}

pub struct Devices {
    set_r: DeviceSet<R>,
    set_gnd: DeviceSet<Gnd>,
}

impl Default for Devices {
    fn default() -> Self {
        Devices{ set_r: DeviceSet::new(), set_gnd: DeviceSet::new() }
    }
}

impl Drawable for Devices {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in self.iter_device_traits() {
            let vct_c = d.borrow().compose_transform(vct);
            d.borrow().draw_persistent(vct_c, vcscale, frame);
        }
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in self.iter_device_traits().iter().filter(|&d| d.borrow().get_interactable().selected) {
            let vct_c = d.borrow().compose_transform(vct);
            d.borrow().draw_selected(vct_c, vcscale, frame);
        }
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in self.iter_device_traits().iter().filter(|&d| d.borrow().get_interactable().tentative) {
            let vct_c = d.borrow().compose_transform(vct);
            d.borrow().draw_preview(vct_c, vcscale, frame);
        }
    }
}

impl Devices {
    pub fn place_res(&mut self) -> Rc<RefCell<dyn DeviceExt>> {
        self.set_r.new_instance()
    }
    pub fn place_gnd(&mut self) -> Rc<RefCell<dyn DeviceExt>> {
        self.set_gnd.new_instance()
    }
    pub fn iter_device_traits(&self) -> Vec<Rc<RefCell<dyn DeviceExt>>> {
        [
            self.set_gnd.devices_traits(),
            self.set_r.devices_traits(),
        ].concat()
    }
    pub fn ports_ssp(&self) -> Vec<SSPoint> {
        self.set_gnd.set.iter().flat_map(|d| d.0.borrow().ports_ssp())
        .chain(self.set_r.set.iter().flat_map(|d| d.0.borrow().ports_ssp()))
        .collect()
    }
    pub fn tentatives_to_selected(&mut self) {
        for d in self.iter_device_traits() {
            d.borrow_mut().tentatives_to_selected();
        }
    }
    pub fn move_selected(&mut self, ssv: Vector2D<i16, SchematicSpace>) {
        for d in self.iter_device_traits() {
            d.borrow_mut().move_selected(ssv);
        }
    }
    pub fn draw_selected_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in self.iter_device_traits() {
            d.borrow_mut().draw_selected_preview(vct, vcscale, frame);
        }
    }
    pub fn clear_selected(&mut self) {
        for d in self.iter_device_traits() {
            d.borrow_mut().clear_selected();
        }
    }
    pub fn clear_tentatives(&mut self) {
        for d in self.iter_device_traits() {
            d.borrow_mut().clear_tentatives();
        }
    }
    pub fn bounding_box(&self) -> VSBox {
        let vt = self.iter_device_traits();
        let pts = vt.iter()
        .flat_map(
            |d| 
            [d.borrow().bounds().min, d.borrow().bounds().max].into_iter()
        );
        SSBox::from_points(pts).cast().cast_unit()
    }
    pub fn delete_selected(&mut self) {
        todo!()
    }
    pub fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        for d in self.iter_device_traits() {
            if d.borrow().ports_occupy_ssp(ssp) {return true}
        }
        false
    }
}


