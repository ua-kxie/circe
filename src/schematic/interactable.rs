//! common functionality for interactive schematic elements

use crate::transforms::{SSBox, SSPoint, SSTransform, SchematicSpace};

/// trait to facillitates and unify implementation of interactive logic
pub trait Interactive {
    fn transform(&mut self, sst: SSTransform);
}

/// struct to facillitates and unify implementation of interactive logic through composition
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct Interactable {
    /// the bounds of the interactable. e.g. mouse hover over this area should highlight the interactable.
    pub bounds: SSBox,
}

impl Interactable {
    pub fn new() -> Self {
        Interactable {
            bounds: SSBox::default(),
        }
    }
    /// returns true if Schematic Space Point intersects with bounds.
    pub fn contains_ssp(&self, ssp: SSPoint) -> bool {
        let mut ssb = self.bounds;
        ssb.set_size(ssb.size() + euclid::Size2D::<i16, SchematicSpace>::new(1, 1));
        ssb.contains(ssp)
    }
    /// returns true if bounds intersects with argument.
    pub fn intersects_ssb(&self, ssb: &SSBox) -> bool {
        self.bounds.intersects(ssb)
    }
}
