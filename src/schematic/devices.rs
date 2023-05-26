// ex: Vgnd0 net1 0 0
// device Id, net at port, ground net '0', device voltage 0
mod devicetype;
mod deviceinstance;

use std::{rc::Rc, cell::RefCell};
use euclid::Vector2D;
use iced::widget::canvas::Frame;

use crate::{
    schematic::nets::{Drawable},
    transforms::{
        SSPoint, VSBox, VCTransform, SchematicSpace, SSBox
    }, 
};

pub use self::deviceinstance::DeviceInstance;
use self::devicetype::DeviceType;

pub struct Devices {
    devices_vec: Vec<Rc<RefCell<DeviceInstance>>>,

    res: Rc<DeviceType>,
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


