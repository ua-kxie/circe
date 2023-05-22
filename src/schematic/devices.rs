// ex: Vgnd0 net1 0 0
// device Id, net at port, ground net '0', device voltage 0
mod devicetype;
mod deviceinstance;

use std::sync::Arc;
use iced::widget::canvas::Frame;

use crate::{
    schematic::nets::{Drawable},
    transforms::{
        SSPoint, VSBox, VCTransform
    }, 
};

pub use self::deviceinstance::DeviceInstance;
use self::devicetype::DeviceType;

#[derive(Debug)]
pub struct Devices {
    devices_vec: Vec<Arc<DeviceInstance>>,

    res: Arc<DeviceType>,
}

impl Default for Devices {
    fn default() -> Self {
        Devices::new()
    }
}

impl Drawable for Devices {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.devices_vec {
            d.draw_persistent(vct, vcscale, frame);
        }
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.devices_vec {
            if d.selected() {
                d.draw_selected(vct, vcscale, frame);
            }
        }
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.devices_vec {
            d.draw_preview(vct, vcscale, frame);
        }
    }
}

impl Devices {
    pub fn clear_selected(&mut self) {
        for d in &self.devices_vec {
            d.unset_select();
        }
    }
    pub fn bounding_box(&self) -> VSBox {
        let pts = self.devices_vec.iter().flat_map(|i| [i.bounds().min, i.bounds().max].into_iter());
        VSBox::from_points(pts)
    }
    pub fn push(&mut self, di: DeviceInstance) {
        self.devices_vec.push(Arc::new(di));
    }
    pub fn iter(&self) -> std::slice::Iter<Arc<DeviceInstance>> {
        self.devices_vec.iter()
    }
    pub fn place_res(&mut self, ssp: SSPoint) -> DeviceInstance {
        DeviceInstance::new_res(ssp, self.res.clone())
    }
    pub fn delete_selected(&mut self) {
        self.devices_vec = self.devices_vec.iter().filter_map(|e| {
            if !e.selected() {Some(e.clone())} else {None}
        }).collect()
    }
    fn new() -> Self {
        Devices { devices_vec: vec![], res: Arc::new(DeviceType::new_res()) }
    }
}


