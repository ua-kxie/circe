//! common functionality for interactive schematic elements
use crate::transforms::{SSBox, SSPoint, SchematicSpace, VSBox, VSBoxExt, VSPoint, VVTransform};

/// trait to facillitates and unify implementation of interactive logic
pub trait Interactive {
    fn transform(&mut self, sst: VVTransform);
}

/// struct to facillitates and unify implementation of interactive logic through composition
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Interactable {
    /// the bounds of the interactable. e.g. mouse hover over this area should highlight the interactable.
    pub bounds: VSBox,
}

impl Interactable {
    pub fn new(vsb: VSBox) -> Self {
        Interactable { bounds: vsb }
    }
    /// returns true if Schematic Space Point intersects with bounds.
    pub fn contains_ssp(&self, ssp: SSPoint) -> bool {
        let mut ssb: SSBox = self.bounds.round_out().cast().cast_unit();
        ssb.set_size(ssb.size() + euclid::Size2D::<i16, SchematicSpace>::new(1, 1));
        ssb.contains(ssp)
    }
    /// returns true if Viewport Space Point intersects with bounds.
    pub fn contains_vsp(&self, vsp: VSPoint) -> bool {
        self.bounds.inclusive_contains(vsp)
    }
    /// returns true if bounds intersects with argument.
    pub fn intersects_vsb(&self, vsb: &VSBox) -> bool {
        self.bounds.intersects(vsb)
    }
}
