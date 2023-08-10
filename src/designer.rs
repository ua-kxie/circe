//! Designer
//! Concrete types for schematic content for designing device appearances
//! intended to eventually allow users to define hierarchical devices
//! for now, intended only to allow devs to quickly draw up basic device symbols

use crate::schematic::devices::port::{Port, RcRPort};
use crate::schematic::interactable::Interactive;
use crate::schematic::{self, SchematicElement, SchematicMsg};
use crate::transforms::{Point, SSBox, SSPoint, VSPoint};
use crate::{
    transforms::{VCTransform, VSBox, VVTransform},
    viewport::Drawable,
};
use iced::widget::canvas::path::Builder;
use iced::widget::canvas::{event::Event, Frame};
use iced::widget::canvas::{stroke, LineCap, Stroke};
use iced::Color;
use send_wrapper::SendWrapper;

use crate::schematic::devices::strokes::{Bounds, Linear, RcRBounds, RcRLinear};
use std::collections::HashSet;

/// an enum to unify different types in schematic (lines and ellipses)
#[derive(Debug, Clone)]
pub enum DesignerElement {
    Linear(RcRLinear),
    Port(RcRPort),
    Bounds(RcRBounds),
}

impl PartialEq for DesignerElement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Linear(l0), Self::Linear(r0)) => {
                by_address::ByAddress(l0.0.clone()) == by_address::ByAddress(r0.0.clone())
            }
            (Self::Port(l0), Self::Port(r0)) => {
                by_address::ByAddress(l0.0.clone()) == by_address::ByAddress(r0.0.clone())
            }
            (Self::Bounds(l0), Self::Bounds(r0)) => {
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
            DesignerElement::Bounds(rcrb) => by_address::ByAddress(rcrb.0.clone()).hash(state),
        }
    }
}

impl Drawable for DesignerElement {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            DesignerElement::Linear(l) => l.0.borrow().draw_persistent(vct, vcscale, frame),
            DesignerElement::Port(l) => l.0.borrow().draw_persistent(vct, vcscale, frame),
            DesignerElement::Bounds(l) => l.0.borrow().draw_persistent(vct, vcscale, frame),
        }
    }

    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            DesignerElement::Linear(l) => l.0.borrow().draw_selected(vct, vcscale, frame),
            DesignerElement::Port(l) => l.0.borrow().draw_selected(vct, vcscale, frame),
            DesignerElement::Bounds(l) => l.0.borrow().draw_selected(vct, vcscale, frame),
        }
    }

    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        match self {
            DesignerElement::Linear(l) => l.0.borrow().draw_preview(vct, vcscale, frame),
            DesignerElement::Port(l) => l.0.borrow().draw_preview(vct, vcscale, frame),
            DesignerElement::Bounds(l) => l.0.borrow().draw_preview(vct, vcscale, frame),
        }
    }
}

impl SchematicElement for DesignerElement {
    fn contains_vsp(&self, vsp: VSPoint) -> bool {
        match self {
            DesignerElement::Linear(l) => l.0.borrow().interactable.contains_vsp(vsp),
            DesignerElement::Port(l) => l.0.borrow().interactable.contains_vsp(vsp),
            DesignerElement::Bounds(l) => l.0.borrow().interactable.contains_vsp(vsp),
        }
    }
}

impl DesignerElement {
    fn bounding_box(&self) -> VSBox {
        match self {
            DesignerElement::Linear(l) => l.0.borrow().interactable.bounds,
            DesignerElement::Port(p) => p.0.borrow().interactable.bounds,
            DesignerElement::Bounds(p) => p.0.borrow().interactable.bounds,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    CanvasEvent(Event),
    Line,
}

impl schematic::ContentMsg for Msg {
    fn canvas_event_msg(event: Event) -> Self {
        Msg::CanvasEvent(event)
    }
}

#[derive(Debug, Clone, Default)]
pub enum DesignerSt {
    #[default]
    Idle,
    Line(Option<(VSPoint, VSPoint)>),
    Bounds(Option<(SSPoint, SSPoint)>),
}

/// struct holding schematic state (lines and ellipses)
#[derive(Debug, Clone)]
pub struct Designer {
    pub infobarstr: Option<String>,

    state: DesignerSt,

    content: HashSet<DesignerElement>,

    rounding_interval: f32,
    curpos_vsp: VSPoint,
}

impl Default for Designer {
    fn default() -> Self {
        Self {
            infobarstr: Default::default(),
            state: Default::default(),
            content: Default::default(),
            rounding_interval: 0.25,
            curpos_vsp: Default::default(),
        }
    }
}

impl Designer {
    fn update_cursor_vsp(&mut self, curpos_vsp: VSPoint) {
        self.curpos_vsp = (curpos_vsp / self.rounding_interval).round() * self.rounding_interval;
        match &mut self.state {
            DesignerSt::Bounds(opt_vsps) => {
                if let Some((_ssp0, ssp1)) = opt_vsps {
                    *ssp1 = self.curpos_vsp.round().cast().cast_unit();
                }
            }
            DesignerSt::Line(Some((_vsp0, vsp1))) => {
                *vsp1 = self.curpos_vsp;
            }
            DesignerSt::Idle => {}
            _ => {}
        }
    }
    fn occupies_vsp(&self, _vsp: VSPoint) -> bool {
        false
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
        fn draw_snap_marker(vsp: VSPoint, vct: VCTransform, _vcscale: f32, frame: &mut Frame) {
            let cursor_stroke = || -> Stroke {
                Stroke {
                    width: 1.0,
                    style: stroke::Style::Solid(Color::from_rgb(1.0, 0.9, 0.0)),
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                }
            };
            let dim = 0.25;
            let x0 = vsp.x - dim;
            let x1 = vsp.x + dim;
            let y0 = vsp.y - dim;
            let y1 = vsp.y + dim;
            let mut path_builder = Builder::new();
            path_builder.move_to(Point::from(vct.transform_point(VSPoint::new(x0, vsp.y))).into());
            path_builder.line_to(Point::from(vct.transform_point(VSPoint::new(x1, vsp.y))).into());
            path_builder.move_to(Point::from(vct.transform_point(VSPoint::new(vsp.x, y0))).into());
            path_builder.line_to(Point::from(vct.transform_point(VSPoint::new(vsp.x, y1))).into());
            frame.stroke(&path_builder.build(), cursor_stroke());
        }
        match &self.state {
            DesignerSt::Bounds(opt_vsps) => {
                draw_snap_marker(self.curpos_vsp.round(), vct, vcscale, frame);
                if let Some((ssp0, ssp1)) = opt_vsps {
                    Bounds::new(SSBox::from_points([ssp0, ssp1])).draw_preview(vct, vcscale, frame);
                }
            }
            DesignerSt::Line(opt_vsps) => {
                if let Some((vsp0, vsp1)) = opt_vsps {
                    Linear::new(*vsp0, *vsp1).draw_preview(vct, vcscale, frame);
                }
            }
            DesignerSt::Idle => {}
        }
    }
}

impl schematic::Content<DesignerElement, Msg> for Designer {
    fn curpos_update(&mut self, vsp: VSPoint) {
        self.update_cursor_vsp(vsp);
    }
    fn curpos_vsp(&self) -> VSPoint {
        self.curpos_vsp
    }
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
    fn intersects_vsb(&mut self, vsb: VSBox) -> HashSet<DesignerElement> {
        let mut ret = HashSet::new();
        for d in &self.content {
            match d {
                DesignerElement::Linear(l) => {
                    if l.0.borrow_mut().interactable.intersects_vsb(&vsb) {
                        ret.insert(DesignerElement::Linear(l.clone()));
                    }
                }
                DesignerElement::Port(l) => {
                    if l.0.borrow_mut().interactable.intersects_vsb(&vsb) {
                        ret.insert(DesignerElement::Port(l.clone()));
                    }
                }
                DesignerElement::Bounds(l) => {
                    if l.0.borrow_mut().interactable.intersects_vsb(&vsb) {
                        ret.insert(DesignerElement::Bounds(l.clone()));
                    }
                }
            }
        }
        ret
    }

    /// returns the first CircuitElement after skip which intersects with curpos_ssp, if any.
    /// count is updated to track the number of elements skipped over
    fn selectable(
        &mut self,
        vsp: VSPoint,
        skip: usize,
        count: &mut usize,
    ) -> Option<DesignerElement> {
        for d in &self.content {
            match d {
                DesignerElement::Linear(l) => {
                    if l.0.borrow_mut().interactable.contains_vsp(vsp) {
                        if *count == skip {
                            // skipped just enough
                            return Some(d.clone());
                        } else {
                            *count += 1;
                        }
                    }
                }
                DesignerElement::Port(l) => {
                    if l.0.borrow_mut().interactable.contains_vsp(vsp) {
                        if *count == skip {
                            // skipped just enough
                            return Some(d.clone());
                        } else {
                            *count += 1;
                        }
                    }
                }
                DesignerElement::Bounds(b) => {
                    if b.0.borrow_mut().interactable.contains_vsp(vsp) {
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
            Msg::CanvasEvent(event) => {
                let mut state = self.state.clone();
                let mut ret_msg_tmp = SchematicMsg::None;
                match (&mut state, event) {
                    // draw bounds
                    (
                        DesignerSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::B,
                            modifiers: _,
                        }),
                    ) => {
                        state = DesignerSt::Bounds(None);
                    }
                    (
                        DesignerSt::Bounds(opt_ws),
                        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
                    ) => {
                        let new_st;
                        if let Some((ssp0, ssp1)) = opt_ws {
                            // subsequent click
                            if self.curpos_vsp.round().cast().cast_unit() == *ssp0 {
                                new_st = DesignerSt::Idle; // zero size bounds: do not make
                            } else {
                                self.content.insert(DesignerElement::Bounds(RcRBounds::new(
                                    Bounds::new(SSBox::from_points([ssp0, ssp1])),
                                )));
                                new_st = DesignerSt::Idle; // created a valid bound: return to idle state
                            }
                            ret_msg_tmp = SchematicMsg::ClearPassive;
                        } else {
                            // first click
                            let ssp = self.curpos_vsp.round().cast().cast_unit();
                            new_st = DesignerSt::Bounds(Some((ssp, ssp)));
                        }
                        state = new_st;
                    }
                    // port placement
                    (
                        DesignerSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::P,
                            modifiers: _,
                        }),
                    ) => {
                        ret_msg_tmp = SchematicMsg::NewElement(SendWrapper::new(
                            DesignerElement::Port(RcRPort::new(Port::default())),
                        ));
                    }
                    // line placement
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
                        let vsp = self.curpos_vsp;
                        let new_ws;
                        if let Some((ssp0, _ssp1)) = opt_ws {
                            // subsequent click
                            if vsp == *ssp0 {
                                new_ws = None;
                            } else if self.occupies_vsp(vsp) {
                                self.content.insert(DesignerElement::Linear(RcRLinear::new(
                                    Linear::new(*ssp0, vsp),
                                )));
                                new_ws = None;
                            } else {
                                self.content.insert(DesignerElement::Linear(RcRLinear::new(
                                    Linear::new(*ssp0, vsp),
                                )));
                                new_ws = Some((vsp, vsp));
                            }
                            ret_msg_tmp = SchematicMsg::ClearPassive;
                        } else {
                            // first click
                            new_ws = Some((vsp, vsp));
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

    fn move_elements(&mut self, elements: &HashSet<DesignerElement>, sst: &VVTransform) {
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
                DesignerElement::Bounds(l) => {
                    l.0.borrow_mut().transform(*sst);
                    // if moving an existing line, does nothing
                    // inserts the line if placing a new line
                    self.content.insert(DesignerElement::Bounds(l.clone()));
                }
            }
        }
    }

    fn copy_elements(&mut self, elements: &HashSet<DesignerElement>, sst: &VVTransform) {
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
                DesignerElement::Bounds(rcl) => {
                    //unwrap refcell
                    let refcell_d = rcl.0.borrow();
                    let mut bounds = (*refcell_d).clone();
                    bounds.transform(*sst);

                    //build BaseElement
                    self.content
                        .insert(DesignerElement::Bounds(RcRBounds::new(bounds)));
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
