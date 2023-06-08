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
    pub fn tentative_by_ssp(&mut self, ssp: SSPoint) {
        if self.bounds.contains(ssp) {
            self.tentative = true;
        }
    }
}

pub trait Interactive {
    fn transform(&mut self, sst: Transform2D<i16, SchematicSpace, SchematicSpace>);
}

