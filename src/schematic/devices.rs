use std::{rc::Rc, cell::RefCell, hash::Hasher, collections::HashSet};

mod params;
mod devicetype;
mod deviceinstance;

use super::{SchematicSet, BaseElement};
use devicetype::{DeviceClass, r::R, gnd::Gnd, v::V};
use deviceinstance::Device;
use crate::{
    schematic::Drawable,
    transforms::{
        SSPoint, VSBox, VCTransform, SSBox
    }, 
};

use iced::widget::canvas::Frame;
use by_address::ByAddress;

#[derive(Debug, Clone)]
pub struct RcRDevice (pub Rc<RefCell<Device>>);

impl PartialEq for RcRDevice {
    fn eq(&self, other: &Self) -> bool {
        ByAddress(self.0.clone()) == ByAddress(other.0.clone())
    }
}
impl Eq for RcRDevice{}
impl std::hash::Hash for RcRDevice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ByAddress(self.0.clone()).hash(state);
    }
}

#[derive(Debug)]
struct ClassManager {
    wm: usize,
}

impl ClassManager {
    pub fn new() -> Self {
        ClassManager { wm: 0 }
    }
    pub fn incr(&mut self) -> usize {
        self.wm += 1;
        self.wm
    }
}

#[derive(Debug)]
struct DevicesManager {
    gnd: ClassManager,
    r: ClassManager,
    v: ClassManager,
}

impl Default for DevicesManager {
    fn default() -> Self {
        Self { 
            gnd: ClassManager::new(), 
            r: ClassManager::new(), 
            v: ClassManager::new(), 
        }
    }
}

#[derive(Debug, Default)]
pub struct Devices {
    set: HashSet<RcRDevice>, 
    manager: DevicesManager,
}

impl Drawable for Devices {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.set {
            d.0.borrow().draw_persistent(vct, vcscale, frame);
        }
    }
    fn draw_selected(&self, _vct: VCTransform, _vcscale: f32, _frame: &mut Frame) {
        panic!("not intended for use");
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in self.set.iter().filter(|&d| d.0.borrow().interactable.tentative) {
            d.0.borrow().draw_preview(vct, vcscale, frame);
        }
    }
}

impl Devices {
    pub fn op(&mut self, pkvecvaluesall: &paprika::PkVecvaluesall) {
        for d in &self.set {
            d.0.borrow_mut().op(pkvecvaluesall);
        }
    }
    pub fn insert(&mut self, d: RcRDevice) {
        if !self.set.contains(&d) {
            let ord = match d.0.borrow().class() {
                DeviceClass::Gnd(_) => self.manager.gnd.incr(),
                DeviceClass::R(_) => self.manager.r.incr(),
                DeviceClass::V(_) => self.manager.v.incr(),
            };
            d.0.borrow_mut().set_wm(ord);
            self.set.insert(d);
        }
    }

    pub fn tentatives(&self) -> impl Iterator<Item = RcRDevice> + '_ {
        self.set.iter().filter_map(
            |x| 
            if x.0.borrow().interactable.tentative {
                Some(x.clone())
            } else {
                None
            }
        )
    }
    pub fn tentatives_by_ssbox(&mut self, ssb: &SSBox) {
        let _: Vec<_> = self.set.iter().map(|d| {
            // d.0.borrow_mut().tentative_by_vsb(vsb);
            d.0.borrow_mut().interactable.tentative_by_ssb(ssb);
        }).collect();
    }
    pub fn new_res(&mut self) -> RcRDevice {
        let d = Device::new_with_ord_class(0, DeviceClass::R(R::new()));
        RcRDevice(Rc::new(RefCell::new(d)))
    }
    pub fn new_gnd(&mut self) -> RcRDevice {
        let d = Device::new_with_ord_class(0, DeviceClass::Gnd(Gnd::new()));
        RcRDevice(Rc::new(RefCell::new(d)))
    }
    pub fn new_vs(&mut self) -> RcRDevice {
        let d = Device::new_with_ord_class(0, DeviceClass::V(V::new()));
        RcRDevice(Rc::new(RefCell::new(d)))
    }
    pub fn ports_ssp(&self) -> Vec<SSPoint> {
        self.set.iter()
        .flat_map(|d| d.0.borrow().ports_ssp())
        .collect()
    }
    pub fn clear_tentatives(&mut self) {
        for d in &self.set {
            d.0.borrow_mut().interactable.tentative = false;
        }
    }
    pub fn bounding_box(&self) -> VSBox {
        let pts = self.set.iter()
        .flat_map(
            |d|
            [d.0.borrow().interactable.bounds.min, d.0.borrow().interactable.bounds.max].into_iter()
        );
        SSBox::from_points(pts).cast().cast_unit()
    }
    pub fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        for d in &self.set {
            if d.0.borrow().ports_occupy_ssp(ssp) {return true}
        }
        false
    }
    pub fn delete_device(&mut self, d: &RcRDevice) {
        self.set.remove(d);
    }
    pub fn get_set(&self) -> &HashSet<RcRDevice> {
        &self.set
    }
}

impl SchematicSet for Devices {
    fn selectable(&mut self, curpos_ssp: SSPoint, skip: &mut usize, count: &mut usize) -> Option<BaseElement> {
        for d in &self.set {
            if d.0.borrow_mut().interactable.contains_ssp(curpos_ssp) {
                *count += 1;
                if *count > *skip {
                    *skip = *count;
                    return Some(BaseElement::Device(d.clone()));
                }
            }
        }
        None
    }
}
