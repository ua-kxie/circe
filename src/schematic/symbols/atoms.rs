use crate::schematic::atoms::RcRPort;

use crate::schematic::SchematicAtom;
use crate::transforms::VSBox;
use crate::transforms::VSPoint;

use enum_dispatch::enum_dispatch;

use crate::schematic::atoms::{RcRBounds, RcRCirArc, RcRLineSeg};

/// an enum to unify different types in schematic (lines and ellipses)
#[enum_dispatch(SchematicAtom, Drawable)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DesignerElement {
    RcRLineSeg,
    RcRPort,
    RcRCirArc,
    RcRBounds,
}

// impl PartialEq for DesignerElement {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Self::Linear(l0), Self::Linear(r0)) => {
//                 by_address::ByAddress(l0.0.clone()) == by_address::ByAddress(r0.0.clone())
//             }
//             (Self::CirArc(l0), Self::CirArc(r0)) => {
//                 by_address::ByAddress(l0.0.clone()) == by_address::ByAddress(r0.0.clone())
//             }
//             (Self::Port(l0), Self::Port(r0)) => {
//                 by_address::ByAddress(l0.0.clone()) == by_address::ByAddress(r0.0.clone())
//             }
//             (Self::Bounds(l0), Self::Bounds(r0)) => {
//                 by_address::ByAddress(l0.0.clone()) == by_address::ByAddress(r0.0.clone())
//             }
//             _ => false,
//         }
//     }
// }

// impl Eq for DesignerElement {}

// impl std::hash::Hash for DesignerElement {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         match self {
//             DesignerElement::Linear(rcrl) => by_address::ByAddress(rcrl.0.clone()).hash(state),
//             DesignerElement::CirArc(rcrl) => by_address::ByAddress(rcrl.0.clone()).hash(state),
//             DesignerElement::Port(rcrp) => by_address::ByAddress(rcrp.0.clone()).hash(state),
//             DesignerElement::Bounds(rcrb) => by_address::ByAddress(rcrb.0.clone()).hash(state),
//         }
//     }
// }

// impl Drawable for DesignerElement {
//     fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
//         match self {
//             DesignerElement::Linear(l) => l.0.borrow().draw_persistent(vct, vcscale, frame),
//             DesignerElement::CirArc(l) => l.0.borrow().draw_persistent(vct, vcscale, frame),
//             DesignerElement::Port(l) => l.0.borrow().draw_persistent(vct, vcscale, frame),
//             DesignerElement::Bounds(l) => l.0.borrow().draw_persistent(vct, vcscale, frame),
//         }
//     }

//     fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
//         match self {
//             DesignerElement::Linear(l) => l.0.borrow().draw_selected(vct, vcscale, frame),
//             DesignerElement::CirArc(l) => l.0.borrow().draw_selected(vct, vcscale, frame),
//             DesignerElement::Port(l) => l.0.borrow().draw_selected(vct, vcscale, frame),
//             DesignerElement::Bounds(l) => l.0.borrow().draw_selected(vct, vcscale, frame),
//         }
//     }

//     fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
//         match self {
//             DesignerElement::Linear(l) => l.0.borrow().draw_preview(vct, vcscale, frame),
//             DesignerElement::CirArc(l) => l.0.borrow().draw_preview(vct, vcscale, frame),
//             DesignerElement::Port(l) => l.0.borrow().draw_preview(vct, vcscale, frame),
//             DesignerElement::Bounds(l) => l.0.borrow().draw_preview(vct, vcscale, frame),
//         }
//     }
// }

// impl SchematicAtom for DesignerElement {
//     fn contains_vsp(&self, vsp: VSPoint) -> bool {
//         match self {
//             DesignerElement::Linear(l) => l.0.borrow().interactable.contains_vsp(vsp),
//             DesignerElement::CirArc(l) => l.0.borrow().interactable.contains_vsp(vsp),
//             DesignerElement::Port(l) => l.0.borrow().interactable.contains_vsp(vsp),
//             DesignerElement::Bounds(l) => l.0.borrow().interactable.contains_vsp(vsp),
//         }
//     }
// }

// impl DesignerElement {
//     pub fn bounding_box(&self) -> VSBox {
//         match self {
//             DesignerElement::Linear(l) => l.0.borrow().interactable.bounds,
//             DesignerElement::CirArc(l) => l.0.borrow().interactable.bounds,
//             DesignerElement::Port(p) => p.0.borrow().interactable.bounds,
//             DesignerElement::Bounds(p) => p.0.borrow().interactable.bounds,
//         }
//     }
// }
