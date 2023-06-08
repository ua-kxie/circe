// ex: Vgnd0 net1 0 0
// device Id, net at port, ground net '0', device voltage 0
mod devicetype;
mod deviceinstance;
use devicetype::{Graphics, DeviceClass, r::R, gnd::Gnd};
use deviceinstance::{Device};

use std::{rc::Rc, cell::RefCell, hash::Hasher, collections::HashSet};
use iced::widget::canvas::Frame;

use crate::{
    schematic::nets::{Drawable},
    transforms::{
        SSPoint, VSBox, VCTransform, SchematicSpace, SSBox, VSPoint
    }, 
};

use by_address::ByAddress;

use super::interactable::Interactive;

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

struct ClassManager {
    wm: usize,
    graphics: Vec<Rc<Graphics>>,
}

impl ClassManager {
    pub fn new_w_graphics(graphics: Vec<Rc<Graphics>>) -> Self {
        ClassManager { wm: 0, graphics }
    }
    pub fn incr(&mut self) -> usize {
        self.wm += 1;
        self.wm
    }
}

struct DevicesManager {
    gnd: ClassManager,
    r: ClassManager,
}

impl Default for DevicesManager {
    fn default() -> Self {
        Self { 
            gnd: ClassManager::new_w_graphics(vec![]), 
            r: ClassManager::new_w_graphics(vec![]), 
        }
    }
}

pub struct Devices {
    set: HashSet<RcRDevice>, 
    manager: DevicesManager,
}

impl Default for Devices {
    fn default() -> Self {
        Devices{ set: HashSet::new(), manager: DevicesManager::default() }
    }
}

impl Drawable for Devices {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.set {
            d.0.borrow().draw_persistent(vct, vcscale, frame);
        }
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        panic!("not intended for use");
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in self.set.iter().filter(|&d| d.0.borrow().get_interactable().tentative) {
            d.0.borrow().draw_preview(vct, vcscale, frame);
        }
    }
}

impl Devices {
    pub fn insert(&mut self, d: RcRDevice) {
        if !self.set.contains(&d) {
            let ord = match d.0.borrow().class() {
                DeviceClass::Gnd(_) => self.manager.gnd.incr(),
                DeviceClass::R(_) => self.manager.r.incr(),
            };
            d.0.borrow_mut().set_ord(ord);
            self.set.insert(d);
        }
    }
    pub fn selectable(&self, curpos_ssp: SSPoint, skip: &mut usize, count: &mut usize) -> Option<RcRDevice> {
        for d in &self.set {
            let mut ssb = d.0.borrow().bounds().clone();
            ssb.set_size(ssb.size() + euclid::Size2D::<i16, SchematicSpace>::new(1, 1));
            if ssb.contains(curpos_ssp) {
                *count += 1;
                if *count > *skip {
                    *skip = *count;
                    return Some(d.clone());
                }
            }
        }
        None
    }
    pub fn tentatives(&self) -> impl Iterator<Item = RcRDevice> + '_ {
        self.set.iter().filter_map(
            |x| 
            if x.0.borrow().get_interactable().tentative {
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
    pub fn ports_ssp(&self) -> Vec<SSPoint> {
        self.set.iter()
        .flat_map(|d| d.0.borrow().ports_ssp())
        .collect()
    }
    pub fn clear_tentatives(&mut self) {
        for d in &self.set {
            d.0.borrow_mut().clear_tentatives();
        }
    }
    pub fn bounding_box(&self) -> VSBox {
        let pts = self.set.iter()
        .flat_map(
            |d|
            [d.0.borrow().bounds().min, d.0.borrow().bounds().max].into_iter()
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
}


