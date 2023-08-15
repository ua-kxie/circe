//! net name
//!
//!
//!

// every strongly separated net graph should be assigned one NetLabel on checking ... ?
// or: every net seg holds a Rc<NetLabel>, the same underlying NetLabel if connected
// how to handle multiple user defined within a connected graph? should highlight segments

// maybe just do what cadence does

use std::{cell::RefCell, collections::HashSet, hash::Hasher, rc::Rc};

use iced::{
    widget::canvas::{self, Frame, Text},
    Color, Size,
};

use crate::{
    schematic::interactable::{Interactable, Interactive},
    transforms::{
        sst_to_vvt, vvt_to_sst, Point, SSTransform, VCTransform, VSBox, VSPoint, VSVec, VVTransform,
    },
    viewport::Drawable,
};

use by_address::ByAddress;

/// net label, which can be user set
#[derive(Debug, Clone)]
pub struct NetLabel {
    /// net label
    name: String,

    /// label interactable
    pub interactable: Interactable,
    /// label transform - determines the posisiton and orientation of the label in schematic space
    transform: SSTransform,
    /// interactive bounds before transform
    bounds: VSBox,
}

impl Default for NetLabel {
    fn default() -> Self {
        NetLabel {
            name: String::from("default"),
            interactable: Interactable {
                bounds: VSBox::from_points([
                    VSPoint::origin() - VSVec::new(0.5, 0.5),
                    VSPoint::origin() + VSVec::new(0.5, 0.5),
                ]),
            },
            transform: SSTransform::identity(),
            bounds: VSBox::from_points([
                VSPoint::origin() - VSVec::new(0.5, 0.5),
                VSPoint::origin() + VSVec::new(0.5, 0.5),
            ]),
        }
    }
}

impl NetLabel {
    /// return the user defined net name if it is set, otherwise return the autogenerated net label
    pub fn read(&self) -> &str {
        &self.name
    }

    /// set the user defiend net name
    pub fn set_name(&mut self, newlabel: String) {
        self.name = newlabel;
    }

    /// returns the composite of the device's transform and the given vct
    fn compose_transform(&self, vct: VCTransform) -> VCTransform {
        sst_to_vvt(self.transform).then(&vct)
    }
}

impl Drawable for NetLabel {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let vct_c = self.compose_transform(vct);
        let a = Text {
            content: self.name.clone(),
            position: Point::from(vct_c.transform_point(VSPoint::origin())).into(),
            color: Color::from_rgb(1.0, 1.0, 1.0),
            size: vcscale,
            ..Default::default()
        };
        frame.fill_text(a);

        let f = canvas::Fill {
            style: canvas::Style::Solid(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
            ..canvas::Fill::default()
        };
        let dim = 0.25;
        let ssb = VSBox::new(
            VSPoint::origin() - VSVec::new(dim / 2.0, dim / 2.0),
            VSPoint::origin() + VSVec::new(dim / 2.0, dim / 2.0),
        );

        let csbox = vct_c.outer_transformed_box(&ssb);

        let top_left = csbox.min;
        let size = Size::new(csbox.width(), csbox.height());
        frame.fill_rectangle(Point::from(top_left).into(), size, f);
    }

    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let vct_c = self.compose_transform(vct);
        let a = Text {
            content: self.name.clone(),
            position: Point::from(vct_c.transform_point(VSPoint::origin())).into(),
            color: Color::from_rgb(1.0, 0.8, 0.0),
            size: vcscale,
            ..Default::default()
        };
        frame.fill_text(a);

        let f = canvas::Fill {
            style: canvas::Style::Solid(Color::from_rgba(1.0, 0.8, 0.0, 0.5)),
            ..canvas::Fill::default()
        };
        let dim = 0.25;
        let ssb = VSBox::new(
            VSPoint::origin() - VSVec::new(dim / 2.0, dim / 2.0),
            VSPoint::origin() + VSVec::new(dim / 2.0, dim / 2.0),
        );

        let csbox = vct_c.outer_transformed_box(&ssb);

        let top_left = csbox.min;
        let size = Size::new(csbox.width(), csbox.height());
        frame.fill_rectangle(Point::from(top_left).into(), size, f);
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let vct_c = self.compose_transform(vct);
        let a = Text {
            content: self.name.clone(),
            position: Point::from(vct_c.transform_point(VSPoint::origin())).into(),
            color: Color::from_rgb(1.0, 1.0, 0.5),
            size: vcscale,
            ..Default::default()
        };
        frame.fill_text(a);

        let f = canvas::Fill {
            style: canvas::Style::Solid(Color::from_rgba(1.0, 1.0, 0.5, 0.5)),
            ..canvas::Fill::default()
        };
        let dim = 0.25;
        let ssb = VSBox::new(
            VSPoint::origin() - VSVec::new(dim / 2.0, dim / 2.0),
            VSPoint::origin() + VSVec::new(dim / 2.0, dim / 2.0),
        );

        let csbox = vct_c.outer_transformed_box(&ssb);

        let top_left = csbox.min;
        let size = Size::new(csbox.width(), csbox.height());
        frame.fill_rectangle(Point::from(top_left).into(), size, f);
    }
}

impl Interactive for NetLabel {
    fn transform(&mut self, vvt: VVTransform) {
        let sst = vvt_to_sst(vvt);
        self.transform = self.transform.then(&sst);
        self.interactable.bounds = sst_to_vvt(self.transform).outer_transformed_box(&self.bounds);
    }
}

/// newtype wrapper for `Rc<RefCell<NetLabel>>`. Hashes by memory address.
#[derive(Debug, Clone)]
pub struct RcRLabel(pub Rc<RefCell<NetLabel>>);
impl PartialEq for RcRLabel {
    fn eq(&self, other: &Self) -> bool {
        ByAddress(self.0.clone()) == ByAddress(other.0.clone())
    }
}
impl Eq for RcRLabel {}
impl std::hash::Hash for RcRLabel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ByAddress(self.0.clone()).hash(state);
    }
}

/// struct containing all devices in schematic
#[derive(Debug, Default, Clone)]
pub struct NetLabels {
    /// set of all devices
    set: HashSet<RcRLabel>,
}

impl NetLabels {
    /// returns the first Label after skip which intersects with curpos_ssp in a BaseElement, if any.
    /// count is updated to track the number of elements skipped over
    pub fn selectable(
        &mut self,
        curpos_vsp: VSPoint,
        skip: usize,
        count: &mut usize,
    ) -> Option<RcRLabel> {
        for l in &self.set {
            if l.0.borrow_mut().interactable.contains_vsp(curpos_vsp) {
                if *count == skip {
                    // skipped just enough
                    return Some(l.clone());
                } else {
                    *count += 1;
                }
            }
        }
        None
    }
    /// returns the bounding box of all devices
    pub fn bounding_box(&self) -> VSBox {
        let pts = self.set.iter().flat_map(|l| {
            [
                l.0.borrow().interactable.bounds.min,
                l.0.borrow().interactable.bounds.max,
            ]
            .into_iter()
        });
        VSBox::from_points(pts).cast().cast_unit()
    }
    /// inserts label l into self.
    pub fn insert(&mut self, l: RcRLabel) {
        self.set.insert(l);
    }
    /// return vector of RcRDevice which intersects vsb
    pub fn intersects_vsb(&self, vsb: &VSBox) -> Vec<RcRLabel> {
        let ret: Vec<_> = self
            .set
            .iter()
            .filter_map(|l| {
                if l.0.borrow_mut().interactable.intersects_vsb(vsb) {
                    Some(l.clone())
                } else {
                    None
                }
            })
            .collect();
        ret
    }
    /// return vector of RcRDevice which are contained by vsb
    pub fn contained_by(&self, vsb: &VSBox) -> Vec<RcRLabel> {
        let ret: Vec<_> = self
            .set
            .iter()
            .filter_map(|l| {
                if l.0.borrow_mut().interactable.contained_by(vsb) {
                    Some(l.clone())
                } else {
                    None
                }
            })
            .collect();
        ret
    }
    pub fn delete_item(&mut self, d: &RcRLabel) {
        self.set.remove(d);
    }
    pub fn new_label() -> RcRLabel {
        let l = NetLabel::default();
        RcRLabel(Rc::new(RefCell::new(l)))
    }
}

impl Drawable for RcRLabel {
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

impl Drawable for NetLabels {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for d in &self.set {
            d.0.borrow().draw_persistent(vct, vcscale, frame);
        }
    }
    fn draw_selected(&self, _vct: VCTransform, _vcscale: f32, _frame: &mut Frame) {
        panic!("not intended for use");
    }
    fn draw_preview(&self, _vct: VCTransform, _vcscale: f32, _frame: &mut Frame) {
        panic!("not intended for use");
    }
}
