// ex: Vgnd0 net1 0 0
// device Id, net at port, ground net '0', device voltage 0
mod devicetype;
mod deviceinstance;

use std::sync::Arc;

use euclid::{Size2D, Transform2D, Vector2D, Angle};
use iced::{widget::canvas::{Frame, Stroke, stroke, LineCap, path::Builder, self}, Color, Size};

use crate::{
    schematic::nets::{Selectable, Drawable},
    transforms::{
        SSVec, SSPoint, SSBox, VSBox, SSRect, VSPoint, VCTransform, Point, CanvasSpace, ViewportSpace, CSPoint, CSVec, VSRect, CSBox, CVTransform, VSVec
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
            d.draw_selected(vct, vcscale, frame);
        }
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.devices_vec {
            d.draw_preview(vct, vcscale, frame);
        }
    }
}

impl Devices {
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


