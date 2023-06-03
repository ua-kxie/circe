use crate::transforms::{SSBox, SSVec, VSBox, SSPoint};

#[derive(Debug, Clone, Copy, Default)]
pub struct Interactable {
    pub bounds: SSBox,
    pub tentative: bool,
}

impl Interactable {
    pub fn new() -> Self {
        Interactable { bounds: SSBox::default(), tentative: false }
    }
}

pub trait Interactive {
    fn translate(&mut self, ssv: SSVec);
    fn rotate(&mut self, cw: bool);
    // fn flip(&mut self, vertical: bool);

    fn tentative_by_ssb(&mut self, ssb: &SSBox);
    fn set_translation(&mut self, v: SSPoint);
}

