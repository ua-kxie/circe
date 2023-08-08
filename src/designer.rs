//! Circuit
//! Concrete types for schematic content

use crate::schematic::nets::NetLabels;
use crate::schematic::nets::{NetEdge, NetVertex, Nets, RcRLabel};
use crate::schematic::{
    self, interactable::Interactive, SchematicElement, SchematicMsg,
};
use crate::{
    transforms::{SSBox, SSPoint, SSTransform, VCTransform, VSBox},
    viewport::Drawable,
};
use iced::widget::canvas::{event::Event, Frame};

use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;

/// trait for a type of element in schematic. e.g. nets or devices
pub trait SchematicSet {
    /// returns the first element after skip which intersects with curpos_ssp in a BaseElement, if any.
    /// count is incremented by 1 for every element skipped over
    /// skip is updated if an element is returned, equal to count
    fn selectable(
        &mut self,
        curpos_ssp: SSPoint,
        skip: &mut usize,
        count: &mut usize,
    ) -> Option<DesignerElement>;

    /// returns the bounding box of all contained elements
    fn bounding_box(&self) -> VSBox;
}

/// an enum to unify different types in schematic (nets and devices)
#[derive(Debug, Clone)]
pub enum DesignerElement {
    NetEdge(NetEdge),
    Label(RcRLabel),
}

impl PartialEq for DesignerElement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NetEdge(l0), Self::NetEdge(r0)) => *l0 == *r0,
            (Self::Label(l0), Self::Label(r0)) => {
                by_address::ByAddress(l0) == by_address::ByAddress(r0)
            }
            _ => false,
        }
    }
}

impl Eq for DesignerElement {}

impl std::hash::Hash for DesignerElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DesignerElement::NetEdge(e) => e.hash(state),
            DesignerElement::Label(l) => by_address::ByAddress(l).hash(state),
        }
    }
}

impl Drawable for DesignerElement {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            DesignerElement::NetEdge(e) => e.draw_persistent(vct, vcscale, frame),
            DesignerElement::Label(l) => l.draw_persistent(vct, vcscale, frame),
        }
    }

    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            DesignerElement::NetEdge(e) => e.draw_selected(vct, vcscale, frame),
            DesignerElement::Label(l) => l.draw_selected(vct, vcscale, frame),
        }
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            DesignerElement::NetEdge(e) => e.draw_preview(vct, vcscale, frame),
            DesignerElement::Label(l) => l.draw_preview(vct, vcscale, frame),
        }
    }
}

impl SchematicElement for DesignerElement {
    fn contains_ssp(&self, ssp: SSPoint) -> bool {
        match self {
            DesignerElement::NetEdge(e) => e.interactable.contains_ssp(ssp),
            DesignerElement::Label(l) => l.0.borrow().interactable.contains_ssp(ssp),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    CanvasEvent(Event, SSPoint),
    Wire,
}

impl schematic::ContentMsg for Msg {
    fn canvas_event_msg(event: Event, curpos_ssp: SSPoint) -> Self {
        Msg::CanvasEvent(event, curpos_ssp)
    }
}

#[derive(Debug, Clone, Default)]
pub enum DesignerSt {
    #[default]
    Idle,
    Wiring(Option<(Box<Nets>, SSPoint)>),
}

/// struct holding schematic state (nets, devices, and their locations)
#[derive(Debug, Default, Clone)]
pub struct Designer {
    pub infobarstr: Option<String>,

    state: DesignerSt,

    nets: Nets,
    labels: NetLabels,
    curpos_ssp: SSPoint,
}

impl Designer {
    fn update_cursor_ssp(&mut self, curpos_ssp: SSPoint) {
        self.curpos_ssp = curpos_ssp;
        self.infobarstr = self.nets.net_name_at(curpos_ssp);
        match &mut self.state {
            DesignerSt::Wiring(Some((nets, ssp_prev))) => {
                nets.clear();
                nets.route(*ssp_prev, curpos_ssp);
            }
            DesignerSt::Idle => {}
            _ => {}
        }
    }
}

impl Drawable for Designer {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.nets.draw_persistent(vct, vcscale, frame);
        self.labels.draw_persistent(vct, vcscale, frame);
    }

    fn draw_selected(&self, _vct: VCTransform, _vcscale: f32, _frame: &mut Frame) {
        panic!("not intended for use");
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match &self.state {
            DesignerSt::Wiring(Some((nets, _))) => {
                nets.draw_preview(vct, vcscale, frame);
            }
            DesignerSt::Idle => {}
            _ => {}
        }
    }
}

impl schematic::Content<DesignerElement, Msg> for Designer {
    fn bounds(&self) -> VSBox {
        let bbn = self.nets.bounding_box();
        let bbl = self.labels.bounding_box();
        bbn.union(&bbl)
    }
    fn intersects_ssb(&mut self, ssb: SSBox) -> HashSet<DesignerElement> {
        let mut ret = HashSet::new();
        for seg in self.nets.intersects_ssbox(&ssb) {
            ret.insert(DesignerElement::NetEdge(seg));
        }
        for rcrl in self.labels.intersects_ssb(&ssb) {
            ret.insert(DesignerElement::Label(rcrl));
        }
        ret
    }

    fn occupies_ssp(&self, _ssp: SSPoint) -> bool {
        false
    }

    /// returns the first CircuitElement after skip which intersects with curpos_ssp, if any.
    /// count is updated to track the number of elements skipped over
    fn selectable(
        &mut self,
        ssp: SSPoint,
        skip: usize,
        count: &mut usize,
    ) -> Option<DesignerElement> {
        if let Some(l) = self.labels.selectable(ssp, skip, count) {
            return Some(DesignerElement::Label(l));
        }
        if let Some(e) = self.nets.selectable(ssp, skip, count) {
            return Some(DesignerElement::NetEdge(e));
        }
        None
    }

    fn update(&mut self, msg: Msg) -> SchematicMsg<DesignerElement> {
        let ret_msg = match msg {
            Msg::CanvasEvent(event, curpos_ssp) => {
                if let Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = event {
                    self.update_cursor_ssp(curpos_ssp);
                }

                let mut state = self.state.clone();
                let mut ret_msg_tmp = SchematicMsg::None;
                match (&mut state, event) {
                    // wiring
                    (
                        DesignerSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::W,
                            modifiers: _,
                        }),
                    ) => {
                        state = DesignerSt::Wiring(None);
                    }
                    (
                        DesignerSt::Wiring(opt_ws),
                        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
                    ) => {
                        let ssp = curpos_ssp;
                        let new_ws;
                        if let Some((g, prev_ssp)) = opt_ws {
                            // subsequent click
                            if ssp == *prev_ssp {
                                new_ws = None;
                            } else if self.occupies_ssp(ssp) {
                                self.nets.merge(g.as_ref(), vec![]);
                                new_ws = None;
                            } else {
                                self.nets.merge(g.as_ref(), vec![]);
                                new_ws = Some((Box::<Nets>::default(), ssp));
                            }
                            ret_msg_tmp = SchematicMsg::ClearPassive;
                        } else {
                            // first click
                            new_ws = Some((Box::<Nets>::default(), ssp));
                        }
                        state = DesignerSt::Wiring(new_ws);
                    }
                    // label
                    (
                        DesignerSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::L,
                            modifiers: _,
                        }),
                    ) => {
                        let l = NetLabels::new_label();
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(DesignerElement::Label(l)));
                    }
                    // state reset
                    (
                        _,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Escape,
                            modifiers: _,
                        }),
                    ) => {
                        state = DesignerSt::Idle;
                    }
                    _ => {}
                }
                self.state = state;
                ret_msg_tmp
            }
            Msg::Wire => {
                self.state = DesignerSt::Wiring(None);
                SchematicMsg::None
            }
        };
        ret_msg
    }

    fn move_elements(&mut self, elements: &HashSet<DesignerElement>, sst: &SSTransform) {
        for e in elements {
            match e {
                DesignerElement::NetEdge(e) => {
                    self.nets.transform(e.clone(), *sst);
                }
                DesignerElement::Label(l) => {
                    l.0.borrow_mut().transform(*sst);
                    // if moving an existing label, does nothing
                    // inserts the label if placing a new label
                    self.labels.insert(l.clone());
                }
            }
        }
    }

    fn copy_elements(&mut self, elements: &HashSet<DesignerElement>, sst: &SSTransform) {
        for e in elements {
            match e {
                DesignerElement::NetEdge(seg) => {
                    let mut seg = seg.clone();
                    seg.transform(*sst);
                    self.nets
                        .graph
                        .add_edge(NetVertex(seg.src), NetVertex(seg.dst), seg.clone());
                }
                DesignerElement::Label(rcl) => {
                    //unwrap refcell
                    let refcell_d = rcl.0.borrow();
                    let mut label = (*refcell_d).clone();
                    label.transform(*sst);

                    //build BaseElement
                    let l_refcell = RefCell::new(label);
                    let l_rc = Rc::new(l_refcell);
                    let rcr_label = RcRLabel(l_rc);
                    self.labels.insert(rcr_label);
                }
            }
        }
    }

    fn delete_elements(&mut self, elements: &HashSet<DesignerElement>) {
        for e in elements {
            match e {
                DesignerElement::NetEdge(e) => {
                    self.nets.delete_edge(e);
                }
                DesignerElement::Label(l) => {
                    self.labels.delete_item(l);
                }
            }
        }
    }

    fn is_idle(&self) -> bool {
        matches!(self.state, DesignerSt::Idle)
    }
}
