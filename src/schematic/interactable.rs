use euclid::Transform2D;

use crate::transforms::{SSBox, SchematicSpace, SSPoint};

#[derive(Debug, Clone, Copy, Default)]
pub struct Interactable {
    pub bounds: SSBox,
    pub tentative: bool,
}

impl Interactable {
    pub fn new() -> Self {
        Interactable { bounds: SSBox::default(), tentative: false }
    }
    pub fn tentative_by_ssb(&mut self, ssb: &SSBox) {
        if self.bounds.intersects(ssb) {
            self.tentative = true;
        }
    }
    pub fn contains_ssp(&mut self, ssp: SSPoint) -> bool {
        let mut ssb = self.bounds;
        ssb.set_size(ssb.size() + euclid::Size2D::<i16, SchematicSpace>::new(1, 1));
        ssb.contains(ssp)
    }
}

pub trait Interactive {
    fn transform(&mut self, sst: Transform2D<i16, SchematicSpace, SchematicSpace>);
}

