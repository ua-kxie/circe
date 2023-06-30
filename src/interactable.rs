//! common functionality for interactive schematic elements

use crate::transforms::{SSBox, SSPoint, SSTransform, SchematicSpace};

/// trait to facillitates and unify implementation of interactive logic
pub trait Interactive {
    fn transform(&mut self, sst: SSTransform);
}

/// struct to facillitates and unify implementation of interactive logic through composition
#[derive(Debug, Clone, Copy, Default)]
pub struct Interactable {
    /// the bounds of the interactable. e.g. mouse hover over this area should highlight the interactable.
    pub bounds: SSBox,
    /// tentative flag. If true, marks this interactable as under tentative selection. i.e. mouse hovering over but not yet selected.
    pub tentative: bool,
}

impl Interactable {
    pub fn new() -> Self {
        Interactable {
            bounds: SSBox::default(),
            tentative: false,
        }
    }
    /// sets tentative flag based on Schematic Space Box argument. Set to true if argument intersects with bounds.
    pub fn tentative_by_ssb(&mut self, ssb: &SSBox) {
        self.tentative = self.bounds.intersects(ssb);
    }
    /// returns true if Schematic Space Point intersects with bounds.
    pub fn contains_ssp(&self, ssp: SSPoint) -> bool {
        let mut ssb = self.bounds;
        ssb.set_size(ssb.size() + euclid::Size2D::<i16, SchematicSpace>::new(1, 1));
        ssb.contains(ssp)
    }
}
