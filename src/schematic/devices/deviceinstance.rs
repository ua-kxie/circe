use std::hash::Hasher;

use super::devicetype::{DeviceClass, r::ParamEditor};

use iced::{widget::canvas::{Frame, Text}, Color, Element};

use crate::{
    schematic::{nets::Drawable, interactable::Interactive, Nets},
    transforms::{
        SSPoint, VSPoint, VCTransform, Point, SSTransform, ViewportSpace, sst_to_xxt
    }, 
};
use crate::schematic::interactable::Interactable;
use std::hash::Hash;
#[derive(Debug)]
pub struct Identifier {
    id_prefix: &'static str,  // prefix which determines device type in NgSpice
    id: usize,  // avoid changing - otherwise, 
    custom: Option<String>,  // if some, is set by the user - must use this as is for id - if multiple instances have same, both should be highlighted
}
/*
id collision check:
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
        ret.push_str(self.id_prefix);
        if let Some(s) = &self.custom {
            ret.push_str(s);
        } else {
            ret.push_str(&format!("{}", self.id));
        }
        ret
    }
    pub fn new_with_prefix_ord(id_prefix: &'static str , ord: usize) -> Self {
        Identifier { id_prefix, id: ord, custom: None }
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

#[derive(Debug)]
pub struct Device  {
    id: Identifier,
    pub interactable: Interactable,
    transform: SSTransform,
    class: DeviceClass,
    nets: Vec<String>,
    op: Vec<f32>,
}
impl Device {
    pub fn param_editor(&mut self) -> Option<impl ParamEditor + Into<Element<()>>> {
        self.class.param_editor()
    }
    pub fn set_ord(&mut self, ord: usize) {
        self.id.id = ord;
    }
    pub fn class(&self) -> &DeviceClass {
        &self.class
    }
    pub fn class_mut(&mut self) -> &mut DeviceClass {
        &mut self.class
    }
    pub fn new_with_ord_class(ord: usize, class: DeviceClass) -> Self {
        Device { 
            id: Identifier::new_with_prefix_ord(class.id_prefix(), ord), 
            interactable: Interactable::new(), 
            transform: SSTransform::identity(), 
            class,
            nets: vec![],
            op: vec![],
        }
    }

    pub fn get_interactable(&self) -> Interactable {
        self.interactable
    }
    pub fn clear_tentatives(&mut self) {
        self.interactable.tentative = false;
    }
    pub fn ports_ssp(&self) -> Vec<SSPoint> {
        self.class.graphics().ports().iter().map(|p| self.transform.transform_point(p.offset)).collect()
    }   
    pub fn ports_occupy_ssp(&self, ssp: SSPoint) -> bool {
        for p in self.class.graphics().ports() {
            if self.transform.transform_point(p.offset) == ssp {
                return true;
            }
        }
        false
    }
    pub fn compose_transform(&self, vct: VCTransform) -> VCTransform {
        sst_to_xxt::<ViewportSpace>(self.transform).then(&vct)
    }
    pub fn set_translation(&mut self, v: SSPoint) {
        self.transform.m31 = v.x;
        self.transform.m32 = v.y;
        self.interactable.bounds = self.transform.outer_transformed_box(self.class.graphics().bounds());
    }
    pub fn spice_line(&mut self, nets: &mut Nets) -> String {
        self.nets.clear();
        let mut sline = self.id.ng_id();
        sline.push(' ');
        for p in self.class.graphics().ports() {
            let pt = self.transform.transform_point(p.offset);
            let net = nets.net_at(pt);
            sline.push_str(&net);
            sline.push(' ');
            self.nets.push(net);
        }
        sline.push_str(&self.class.param_summary());
        sline.push('\n');
        sline
    }
    pub fn op(&mut self, pkvecvaluesall: &paprika::PkVecvaluesall) {
        self.op.clear();
        for n in &self.nets {
            for v in &pkvecvaluesall.vecsa {
                if &v.name == n {
                    self.op.push(v.creal as f32);
                    break;
                }
            }
        }
    }
}

impl Drawable for Device {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let vct_c = self.compose_transform(vct);
        self.class.graphics().draw_persistent(vct_c, vcscale, frame);
        
        let a = Text {
            content: self.id.ng_id(),
            position: Point::from(vct_c.transform_point(VSPoint::new(1.0, 1.0))).into(),
            color: Color::from_rgba(1.0, 0.5, 1.0, 1.0),
            size: vcscale,
            ..Default::default()
        };
        frame.fill_text(a);

        let b = Text {
            content: self.class.param_summary(),
            position: Point::from(vct_c.transform_point(VSPoint::new(1.0, 0.0))).into(),
            color: Color::from_rgba(0.5, 1.0, 1.0, 1.0),
            size: vcscale,
            ..Default::default()
        };
        frame.fill_text(b);

        let ports = self.class.graphics().ports();
        for (i, v) in self.op.iter().enumerate() {
            let b = Text {
                content: v.to_string(),
                position: Point::from(vct_c.transform_point(ports[i].offset.cast().cast_unit())).into(),
                color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
                size: vcscale,
                ..Default::default()
            };
            frame.fill_text(b);
        }
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let vct_c = self.compose_transform(vct);
        self.class.graphics().draw_selected(vct_c, vcscale, frame);
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let vct_c = self.compose_transform(vct);
        self.class.graphics().draw_preview(vct_c, vcscale, frame);
    }
}

impl Interactive for Device {
    fn transform(&mut self, sst: SSTransform) {
        self.transform = self.transform.then(&sst);
        self.interactable.bounds = self.transform.outer_transformed_box(self.class.graphics().bounds());
    }
}