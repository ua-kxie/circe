use std::{rc::Rc, hash::Hasher};

use super::devicetype::{Port, Graphics};

use euclid::{Size2D, Transform2D, Vector2D, Angle};
use iced::{widget::canvas::{Frame, Stroke, stroke, LineCap, path::Builder, self, LineDash}, Color, Size};

use crate::{
    schematic::nets::{Selectable, Drawable},
    transforms::{
        SSPoint, VSBox, SSBox, VSPoint, VCTransform, Point, ViewportSpace, SchematicSpace, CanvasSpace
    }, 
};
use std::hash::Hash;
#[derive(Debug, Clone, Copy)]
pub struct Interactable {
    pub bounds: SSBox,
    pub tentative: bool,
    pub selected: bool,
}

impl Interactable {
    fn new() -> Self {
        Interactable { bounds: SSBox::default(), tentative: false, selected: false }
    }
}
#[derive(Debug)]
pub struct Identifier {
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

pub trait DeviceType <T> {
    fn default_graphics() -> Graphics<T>;
}
#[derive(Debug)]
pub struct R;
impl <T> DeviceType<T> for R {
    fn default_graphics() -> Graphics<T> {
        Graphics::default_r()
    }
}
#[derive(Debug)]
pub struct Gnd;
impl <T> DeviceType<T> for Gnd {
    fn default_graphics() -> Graphics<T> {
        Graphics::default_gnd()
    }
}
#[derive(Debug)]
pub struct SingleValue <T> {
    value: f32,
    marker: core::marker::PhantomData<T>,
}
impl <T> SingleValue<T> {
    fn new() -> Self {
        SingleValue { value: 0.0, marker: core::marker::PhantomData }
    }
}
#[derive(Debug)]
pub enum Param <T> {
    Value(SingleValue<T>),
}
#[derive(Debug)]
pub struct Device <T> {
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

pub trait DeviceExt: Drawable {
    fn get_interactable(&self) -> Interactable;
    fn get_transform(&self) -> Transform2D<i16, SchematicSpace, SchematicSpace>;
    fn set_tentative(&mut self);
    fn draw_selected_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
    fn tentative_by_vsb(&mut self, vsb: &VSBox);
    fn tentatives_to_selected(&mut self);
    fn move_selected(&mut self, ssv: Vector2D<i16, SchematicSpace>);
    fn clear_selected(&mut self);
    fn clear_tentatives(&mut self);

    fn ports_ssp(&self) -> Vec<SSPoint>;
    fn ports_occupy_ssp(&self, ssp: SSPoint) -> bool;
    fn stroke_bounds(&self, vct: VCTransform, frame: &mut Frame, stroke: Stroke);
    fn stroke_symbol(&self, vct_composite: VCTransform, frame: &mut Frame, stroke: Stroke);
    fn bounds(&self) -> &SSBox;
    fn set_translation(&mut self, v: SSPoint);
    fn pre_translate(&mut self, ssv: Vector2D<i16, SchematicSpace>);
    fn rotate(&mut self, cw: bool);
    fn compose_transform(&self, vct: VCTransform) -> Transform2D<f32, ViewportSpace, CanvasSpace>;
}
impl <T> DeviceExt for Device<T> {
    fn get_interactable(&self) -> Interactable {
        self.interactable
    }
    fn get_transform(&self) -> Transform2D<i16, SchematicSpace, SchematicSpace> {
        self.transform
    }
    fn set_tentative(&mut self) {
        self.interactable.tentative = true;
    }
    fn draw_selected_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        if self.interactable.selected {
            self.draw_selected(vct, vcscale, frame);
        }
    }
    fn tentative_by_vsb(&mut self, vsb: &VSBox) {
        if self.interactable.bounds.cast().cast_unit().intersects(vsb) {
            self.interactable.tentative = true;
        }
    }
    fn tentatives_to_selected(&mut self) {
        self.interactable.selected = self.interactable.tentative;
        self.interactable.tentative = false;
    }
    fn move_selected(&mut self, ssv: Vector2D<i16, SchematicSpace>) {
        self.pre_translate(ssv.cast_unit());
        self.interactable.selected = false;
    }
    fn clear_selected(&mut self) {
        self.interactable.selected = false;
    }
    fn clear_tentatives(&mut self) {
        self.interactable.tentative = false;
    }
    
    fn ports_ssp(&self) -> Vec<SSPoint> {
        self.graphics.ports().iter().map(|p| self.transform.transform_point(p.offset)).collect()
    }   
    fn ports_occupy_ssp(&self, ssp: SSPoint) -> bool {
        for p in self.graphics.ports() {
            if self.transform.transform_point(p.offset) == ssp {
                return true;
            }
        }
        false
    }
    fn stroke_bounds(&self, vct: VCTransform, frame: &mut Frame, stroke: Stroke) {
        self.graphics.stroke_bounds(vct, frame, stroke);
    }
    fn stroke_symbol(&self, vct: VCTransform, frame: &mut Frame, stroke: Stroke) {
        self.graphics.stroke_symbol(vct, frame, stroke);
    }
    fn bounds(&self) -> &SSBox {
        &self.interactable.bounds
    }
    fn set_translation(&mut self, v: SSPoint) {
        self.transform.m31 = v.x;
        self.transform.m32 = v.y;
        self.interactable.bounds = self.transform.outer_transformed_box(self.graphics.bounds());
    }
    fn pre_translate(&mut self, ssv: Vector2D<i16, SchematicSpace>) {
        self.transform = self.transform.pre_translate(ssv);
        self.interactable.bounds = self.transform.outer_transformed_box(self.graphics.bounds()); //self.device_type.as_ref().get_bounds().cast().cast_unit()
    }
    fn rotate(&mut self, cw: bool) {
        if cw {
            self.transform = self.transform.cast::<f32>().pre_rotate(Angle::frac_pi_2()).cast();
        } else {
            self.transform = self.transform.cast::<f32>().pre_rotate(-Angle::frac_pi_2()).cast();
        }
        self.interactable.bounds = self.transform.cast().outer_transformed_box(&self.graphics.bounds().clone().cast().cast_unit());
    }
    fn compose_transform(&self, vct: VCTransform) -> Transform2D<f32, ViewportSpace, CanvasSpace> {
        self.transform
        .cast()
        .with_destination::<ViewportSpace>()
        .with_source::<ViewportSpace>()
        .then(&vct)
    }
}
impl <T> Drawable for Device<T> {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.graphics.draw_persistent(vct, vcscale, frame);
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.graphics.draw_selected(vct, vcscale, frame);
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.graphics.draw_preview(vct, vcscale, frame);
    }
}

