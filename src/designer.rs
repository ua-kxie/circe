//! Designer
//! Concrete types for schematic content for designing device appearances
//! intended to eventually allow users to define hierarchical devices
//! for now, intended only to allow devs to quickly draw up basic device symbols

use crate::schematic::devices::port::{RcRPort, Port};
use crate::schematic::interactable::Interactive;
use crate::schematic::{self, SchematicElement, SchematicMsg};
use crate::transforms::VSPoint;
use crate::{
    transforms::{SSBox, SSPoint, SSTransform, VCTransform, VSBox},
    viewport::Drawable,
};
use iced::widget::canvas::{event::Event, Frame};
use send_wrapper::SendWrapper;

use crate::schematic::devices::strokes::{Linear, RcRLinear};
use std::collections::HashSet;

/// an enum to unify different types in schematic (lines and ellipses)
#[derive(Debug, Clone)]
pub enum DesignerElement {
    Linear(RcRLinear),
    Port(RcRPort),
}

impl PartialEq for DesignerElement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Linear(l0), Self::Linear(r0)) => {
                by_address::ByAddress(l0.0.clone()) == by_address::ByAddress(r0.0.clone())
            }
            _ => false,
        }
    }
}

impl Eq for DesignerElement {}

impl std::hash::Hash for DesignerElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DesignerElement::Linear(rcrl) => by_address::ByAddress(rcrl.0.clone()).hash(state),
            DesignerElement::Port(rcrp) => by_address::ByAddress(rcrp.0.clone()).hash(state),
        }
    }
}

impl Drawable for DesignerElement {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            DesignerElement::Linear(l) => l.0.borrow().draw_persistent(vct, vcscale, frame),
            DesignerElement::Port(l) => l.0.borrow().draw_persistent(vct, vcscale, frame),
        }
    }

    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            DesignerElement::Linear(l) => l.0.borrow().draw_selected(vct, vcscale, frame),
            DesignerElement::Port(l) => l.0.borrow().draw_selected(vct, vcscale, frame),
        }
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            DesignerElement::Linear(l) => l.0.borrow().draw_preview(vct, vcscale, frame),
            DesignerElement::Port(l) => l.0.borrow().draw_preview(vct, vcscale, frame),
        }
    }
}

impl SchematicElement for DesignerElement {
    fn contains_ssp(&self, ssp: SSPoint) -> bool {
        match self {
            DesignerElement::Linear(l) => l.0.borrow().interactable.contains_ssp(ssp),
            DesignerElement::Port(l) => l.0.borrow().interactable.contains_ssp(ssp),
        }
    }
}

impl DesignerElement {
    fn bounding_box(&self) -> VSBox {
        todo!();
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    CanvasEvent(Event, SSPoint),
    Line,
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
    Line(Option<(SSPoint, SSPoint)>),
}

/// struct holding schematic state (lines and ellipses)
#[derive(Debug, Default, Clone)]
pub struct Designer {
    pub infobarstr: Option<String>,

    state: DesignerSt,

    content: HashSet<DesignerElement>,

    curpos_ssp: SSPoint,
}

impl Designer {
    fn update_cursor_ssp(&mut self, curpos_ssp: SSPoint) {
        self.curpos_ssp = curpos_ssp;
        match &mut self.state {
            DesignerSt::Line(Some((_ssp0, ssp1))) => {
                *ssp1 = curpos_ssp;
            }
            DesignerSt::Idle => {}
            _ => {}
        }
    }
}

impl Drawable for Designer {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        for e in &self.content {
            e.draw_persistent(vct, vcscale, frame);
        }
    }

    fn draw_selected(&self, _vct: VCTransform, _vcscale: f32, _frame: &mut Frame) {
        panic!("not intended for use");
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match &self.state {
            DesignerSt::Line(Some((ssp0, ssp1))) => {
                Linear::new(*ssp0, *ssp1).draw_preview(vct, vcscale, frame)
            }
            DesignerSt::Idle => {}
            _ => {}
        }
    }
}

impl schematic::Content<DesignerElement, Msg> for Designer {
    fn bounds(&self) -> VSBox {
        if !self.content.is_empty() {
            let v_pts: Vec<_> = self
                .content
                .iter()
                .flat_map(|f| [f.bounding_box().min, f.bounding_box().max])
                .collect();
            VSBox::from_points(v_pts)
        } else {
            VSBox::from_points([VSPoint::new(-1.0, -1.0), VSPoint::new(1.0, 1.0)])
        }
    }
    fn intersects_ssb(&mut self, ssb: SSBox) -> HashSet<DesignerElement> {
        let mut ret = HashSet::new();
        for d in &self.content {
            match d {
                DesignerElement::Linear(l) => {
                    if l.0.borrow_mut().interactable.intersects_ssb(&ssb) {
                        ret.insert(DesignerElement::Linear(l.clone()));
                    }
                }
                DesignerElement::Port(l) => {
                    if l.0.borrow_mut().interactable.intersects_ssb(&ssb) {
                        ret.insert(DesignerElement::Port(l.clone()));
                    }
                }
            }
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
        for d in &self.content {
            match d {
                DesignerElement::Linear(l) => {
                    if l.0.borrow_mut().interactable.contains_ssp(ssp) {
                        if *count == skip {
                            // skipped just enough
                            return Some(d.clone());
                        } else {
                            *count += 1;
                        }
                    }
                }
                DesignerElement::Port(l) => {
                    if l.0.borrow_mut().interactable.contains_ssp(ssp) {
                        if *count == skip {
                            // skipped just enough
                            return Some(d.clone());
                        } else {
                            *count += 1;
                        }
                    }
                }
            }
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
                    // port placement
                    (
                        DesignerSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::P,
                            modifiers: _,
                        }),
                    ) => {
                        ret_msg_tmp =
                            SchematicMsg::NewElement(SendWrapper::new(DesignerElement::Port(RcRPort::new(Port::default()))));
                    }
                    // wiring
                    (
                        DesignerSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::W,
                            modifiers: _,
                        }),
                    ) => {
                        state = DesignerSt::Line(None);
                    }
                    (
                        DesignerSt::Line(opt_ws),
                        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
                    ) => {
                        let ssp = curpos_ssp;
                        let new_ws;
                        if let Some((ssp0, _ssp1)) = opt_ws {
                            // subsequent click
                            if ssp == *ssp0 {
                                new_ws = None;
                            } else if self.occupies_ssp(ssp) {
                                self.content.insert(DesignerElement::Linear(RcRLinear::new(
                                    Linear::new(*ssp0, ssp),
                                )));
                                new_ws = None;
                            } else {
                                self.content.insert(DesignerElement::Linear(RcRLinear::new(
                                    Linear::new(*ssp0, ssp),
                                )));
                                new_ws = Some((ssp, ssp));
                            }
                            ret_msg_tmp = SchematicMsg::ClearPassive;
                        } else {
                            // first click
                            new_ws = Some((ssp, ssp));
                        }
                        state = DesignerSt::Line(new_ws);
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
            Msg::Line => {
                self.state = DesignerSt::Line(None);
                SchematicMsg::None
            }
        };
        ret_msg
    }

    fn move_elements(&mut self, elements: &HashSet<DesignerElement>, sst: &SSTransform) {
        for e in elements {
            match e {
                DesignerElement::Linear(l) => {
                    l.0.borrow_mut().transform(*sst);
                    // if moving an existing line, does nothing
                    // inserts the line if placing a new line
                    self.content.insert(DesignerElement::Linear(l.clone()));
                }
                DesignerElement::Port(l) => {
                    l.0.borrow_mut().transform(*sst);
                    // if moving an existing line, does nothing
                    // inserts the line if placing a new line
                    self.content.insert(DesignerElement::Port(l.clone()));
                }
            }
        }
    }

    fn copy_elements(&mut self, elements: &HashSet<DesignerElement>, sst: &SSTransform) {
        for e in elements {
            match e {
                DesignerElement::Linear(rcl) => {
                    //unwrap refcell
                    let refcell_d = rcl.0.borrow();
                    let mut line = (*refcell_d).clone();
                    line.transform(*sst);

                    //build BaseElement
                    self.content
                        .insert(DesignerElement::Linear(RcRLinear::new(line)));
                }
                DesignerElement::Port(rcl) => {
                    //unwrap refcell
                    let refcell_d = rcl.0.borrow();
                    let mut port = (*refcell_d).clone();
                    port.transform(*sst);

                    //build BaseElement
                    self.content
                        .insert(DesignerElement::Port(RcRPort::new(port)));
                }
            }
        }
    }

    fn delete_elements(&mut self, elements: &HashSet<DesignerElement>) {
        for e in elements {
            self.content.remove(e);
        }
    }

    fn is_idle(&self) -> bool {
        matches!(self.state, DesignerSt::Idle)
    }
}
