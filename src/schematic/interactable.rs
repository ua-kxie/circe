use euclid::Transform2D;

use crate::transforms::{SSBox, SSVec, VSBox, SSPoint, ViewportSpace};

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
}

pub trait Interactive {
    fn transform(&mut self, vvt: Transform2D<f32, ViewportSpace, ViewportSpace>);
}

