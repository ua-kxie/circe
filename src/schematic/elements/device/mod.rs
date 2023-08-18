//! devices, e.g. resistors, voltage sources, etc.

use std::{cell::RefCell, hash::Hasher, rc::Rc};

pub mod deviceinstance;
pub mod devicetype;
mod params;

use deviceinstance::Device;

use by_address::ByAddress;

use super::DeviceClass;

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

impl RcRDevice {
    pub fn new_with_ord_class(wm: usize, class: DeviceClass) -> Self {
        RcRDevice(Rc::new(RefCell::new(Device::new_with_ord_class(wm, class))))
    }
}
