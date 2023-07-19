//! devices, e.g. resistors, voltage sources, etc.

use std::{cell::RefCell, collections::HashSet, hash::Hasher, rc::Rc};

mod deviceinstance;
mod devicetype;
mod params;

use crate::{
    transforms::{SSBox, SSPoint, VCTransform, VSBox},
    viewport::Drawable,
};
use deviceinstance::Device;
use devicetype::{gnd::Gnd, r::R, v::V, DeviceClass};

use by_address::ByAddress;
use iced::widget::canvas::Frame;

/// newtype wrapper for `Rc<RefCell<Device>>`. Hashes by memory address.
#[derive(Debug, Clone)]
pub struct RcRDevice(pub Rc<RefCell<Device>>);

impl PartialEq for RcRDevice {
    fn eq(&self, other: &Self) -> bool {
        ByAddress(self.0.clone()) == ByAddress(other.0.clone())
    }
}
impl Eq for RcRDevice {}
impl std::hash::Hash for RcRDevice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ByAddress(self.0.clone()).hash(state);
    }
}

/// struct to keep track of unique IDs for all devices of a type
#[derive(Debug, Clone)]
struct ClassManager {
    // watermark keeps track of the last ID given out
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

/// struct to keep track of unique IDs for all devices of all types
#[derive(Debug, Clone)]
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

/// struct containing all devices in schematic
#[derive(Debug, Default, Clone)]
pub struct Devices {
    /// set of all devices
    set: HashSet<RcRDevice>,
    /// manager to facillate assignment of unique IDs to each device
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
        panic!("not intended for use");
    }
}

impl Devices {
    /// returns the first Device after skip which intersects with curpos_ssp in a BaseElement, if any.
    /// count is updated to track the number of elements skipped over
    pub fn selectable(
        &mut self,
        curpos_ssp: SSPoint,
        skip: &mut usize,
        count: &mut usize,
    ) -> Option<RcRDevice> {
        for d in &self.set {
            if d.0.borrow_mut().interactable.contains_ssp(curpos_ssp) {
                if *count >= *skip {
                    *skip = *count;
                    return Some(d.clone());
                } else {
                    *count += 1;
                }
            }
        }
        None
    }
    /// returns the bounding box of all devices
    pub fn bounding_box(&self) -> VSBox {
        let pts = self.set.iter().flat_map(|d| {
            [
                d.0.borrow().interactable.bounds.min,
                d.0.borrow().interactable.bounds.max,
            ]
            .into_iter()
        });
        SSBox::from_points(pts).cast().cast_unit()
    }
    /// process dc operating point simulation results - draws the voltage of connected nets near the connected port
    pub fn op(&mut self, pkvecvaluesall: &paprika::PkVecvaluesall) {
        for d in &self.set {
            d.0.borrow_mut().op(pkvecvaluesall);
        }
    }
    /// inserts device d into self. Replaces existing if the device already exists.
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
    /// return vector of RcRDevice which intersects ssb
    pub fn intersects_ssb(&self, ssb: &SSBox) -> Vec<RcRDevice> {
        let ret: Vec<_> = self
            .set
            .iter()
            .filter_map(|d| {
                if d.0.borrow_mut().interactable.bounds.intersects(ssb) {
                    Some(d.clone())
                } else {
                    None
                }
            })
            .collect();
        ret
    }
    /// create a new resistor with unique ID
    pub fn new_res(&mut self) -> RcRDevice {
        let d = Device::new_with_ord_class(0, DeviceClass::R(R::new()));
        RcRDevice(Rc::new(RefCell::new(d)))
    }
    /// create a new gnd with unique ID
    pub fn new_gnd(&mut self) -> RcRDevice {
        let d = Device::new_with_ord_class(0, DeviceClass::Gnd(Gnd::new()));
        RcRDevice(Rc::new(RefCell::new(d)))
    }
    /// create a new voltage source with unique ID
    pub fn new_vs(&mut self) -> RcRDevice {
        let d = Device::new_with_ord_class(0, DeviceClass::V(V::new()));
        RcRDevice(Rc::new(RefCell::new(d)))
    }
    /// returns a vector of SSPoints of all coordinates occupied by all ports of all devices. A coordinate is returned once for each port on that coordinate
    pub fn ports_ssp(&self) -> Vec<SSPoint> {
        self.set
            .iter()
            .flat_map(|d| d.0.borrow().ports_ssp())
            .collect()
    }
    pub fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        for d in &self.set {
            if d.0.borrow().ports_occupy_ssp(ssp) {
                return true;
            }
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

impl Drawable for RcRDevice {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.0.borrow().draw_persistent(vct, vcscale, frame);
    }

    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.0.borrow().draw_selected(vct, vcscale, frame);
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.0.borrow().draw_preview(vct, vcscale, frame);
    }
}
