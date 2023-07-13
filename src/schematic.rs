//! Schematic
//! Space in which devices and nets live in

pub(crate) mod devices;
pub(crate) mod nets;

use self::devices::Devices;
pub use self::devices::RcRDevice;
use crate::{interactable, viewport};
use crate::{
    interactable::Interactive,
    transforms::{
        self, CSPoint, Point, SSBox, SSPoint, SSTransform, SSVec, VCTransform, VSBox, VSPoint,
        ViewportSpace,
    },
    viewport::Drawable,
};
use iced::keyboard::Modifiers;
use iced::{
    mouse,
    widget::canvas::{self, event::Event, path::Builder, Frame, LineCap, Stroke},
    Color, Size,
};
use nets::{NetEdge, NetVertex, Nets};
use send_wrapper::SendWrapper;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

pub trait SchematicElement: Hash + Eq + Drawable + Clone {
    // device designer: line, arc
    // circuit: wire, device
    fn contains_ssp(&self, ssp: SSPoint) -> bool;
}

#[derive(Debug, Clone)]
pub enum SchematicMsg<E>
where
    E: SchematicElement,
{
    None,
    NewElement(SendWrapper<E>),
}

pub trait ContentMsg {
    fn canvas_event_msg(event: Event, curpos_ssp: SSPoint) -> Self;
}

#[derive(Debug, Clone)]
pub enum Msg<M, E>
where
    M: ContentMsg,
    E: SchematicElement,
{
    Event(Event, VSPoint),
    SchematicMsg(SchematicMsg<E>),
    ContentMsg(M),
    // NewState(SchematicSt),
}

impl<M, E> viewport::ContentMsg for Msg<M, E>
where
    M: ContentMsg,
    E: SchematicElement,
{
    fn canvas_event_msg(event: Event, curpos_vsp: VSPoint) -> Self {
        Msg::Event(event, curpos_vsp.round().cast().cast_unit())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SchematicSt {
    #[default]
    Idle,
    AreaSelect(SSBox),
    Moving(Option<(SSPoint, SSPoint, SSTransform)>),
    Copying(Option<(SSPoint, SSPoint, SSTransform)>),
    // first click, second click, transform for rotation/flip ONLY
}

pub trait Content<E, M>: Drawable + Default
where
    E: SchematicElement,
{
    // set single tentative flag by curpos_ssp
    fn tentative_by_ssp(&mut self, curpos_ssp: SSPoint);
    // apply sst to elements
    fn move_elements(&mut self, elements: &HashSet<E>, sst: &SSTransform);
    // apply sst to a copy of elements
    fn copy_elements(&mut self, elements: &HashSet<E>, sst: &SSTransform);
    // delete elements
    fn delete_elements(&mut self, elements: &HashSet<E>);
    // ex. wires + devices
    // returns whether or not to clear the passive cache
    fn update(&mut self, msg: M) -> SchematicMsg<E>;
    fn update_cursor_ssp(&mut self, curpos_ssp: SSPoint);
    fn bounds(&self) -> VSBox;
    fn clear_tentatives(&mut self);
    fn tentatives_by_ssbox(&mut self, ssb: SSBox);
    fn tentatives(&self) -> Vec<E>;
    fn occupies_ssp(&self, ssp: SSPoint) -> bool;
    fn delete(&mut self, targets: &HashSet<E>);
    fn transform(&mut self, targets: &HashSet<E>);
    fn selectable(&mut self, ssp: SSPoint, skip: &mut usize, count: &mut usize) -> Option<E>;
    fn set_tentative(&mut self, e: E);
}

/// struct holding schematic state (nets, devices, and their locations)
#[derive(Debug, Clone)]
pub struct Schematic<C, T, M>
where
    C: Content<T, M>,
    T: SchematicElement,
{
    state: SchematicSt,
    pub content: C,
    /// phantom data to mark ContentMsg type
    content_msg: std::marker::PhantomData<M>,
    selskip: usize,
    selected: HashSet<T>,

    curpos_ssp: SSPoint,
}

impl<C, T, M> Default for Schematic<C, T, M>
where
    C: Content<T, M>,
    T: SchematicElement,
{
    fn default() -> Self {
        Self {
            state: Default::default(),
            content: Default::default(),
            selskip: Default::default(),
            selected: Default::default(),
            content_msg: std::marker::PhantomData,
            curpos_ssp: Default::default(),
        }
    }
}

impl<C, E, M> viewport::Content<Msg<M, E>> for Schematic<C, E, M>
where
    M: ContentMsg,
    C: Content<E, M>,
    E: SchematicElement,
{
    fn mouse_interaction(&self) -> mouse::Interaction {
        match self.state {
            SchematicSt::Idle => mouse::Interaction::default(),
            SchematicSt::AreaSelect(_) => mouse::Interaction::Crosshair,
            SchematicSt::Moving(_) => mouse::Interaction::Grabbing,
            SchematicSt::Copying(_) => mouse::Interaction::Grabbing,
        }
    }

    /// draw onto active cache
    fn draw_active(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.content.draw_preview(vct, vcscale, frame);
        match &self.state {
            SchematicSt::Idle => {}
            SchematicSt::AreaSelect(ssb) => {
                // draw the selection area
                let color = if ssb.height() > 0 {
                    Color::from_rgba(1., 1., 0., 0.1) 
                } else {
                    Color::from_rgba(0., 1., 1., 0.1)
                };
                let f = canvas::Fill {
                    style: canvas::Style::Solid(color),
                    ..canvas::Fill::default()
                };
                let csb = vct.outer_transformed_box(&ssb.cast().cast_unit());
                let size = Size::new(csb.width(), csb.height());
                frame.fill_rectangle(Point::from(csb.min).into(), size, f);

                let mut path_builder = Builder::new();
                path_builder.line_to(Point::from(csb.min).into());
                path_builder.line_to(Point::from(CSPoint::new(csb.min.x, csb.max.y)).into());
                path_builder.line_to(Point::from(csb.max).into());
                path_builder.line_to(Point::from(CSPoint::new(csb.max.x, csb.min.y)).into());
                path_builder.line_to(Point::from(csb.min).into());
                let stroke = Stroke {
                    width: (0.1 * vcscale).max(0.1 * 2.0),
                    style: canvas::stroke::Style::Solid(color),
                    line_cap: LineCap::Square,
                    ..Stroke::default()
                };
                frame.stroke(&path_builder.build(), stroke);
            }
            SchematicSt::Moving(Some((ssp0, ssp1, sst))) => {
                // draw selected preview with transform applied
                let vvt = transforms::sst_to_xxt::<ViewportSpace>(sst.then_translate(*ssp1 - *ssp0));

                let vct_c = vvt.then(&vct);
                for be in &self.selected {
                    be.draw_preview(vct_c, vcscale, frame);
                }
            }
            _ => {}
        }
    }
    /// draw onto passive cache
    fn draw_passive(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.content.draw_persistent(vct, vcscale, frame);
        let _: Vec<_> = self
            .selected
            .iter()
            .map(|e| e.draw_selected(vct, vcscale, frame))
            .collect();
    }

    /// returns the bouding box of schematic content
    fn bounds(&self) -> VSBox {
        self.content.bounds()
    }
    /// mutate state based on message and cursor position
    fn update(&mut self, msg: Msg<M, E>) -> bool {
        let mut clear_passive = false;
        // process iced::canvas event
        match msg {
            Msg::Event(event, curpos_csp) => {
                let curpos_ssp = curpos_csp.round().cast().cast_unit();

                if let Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = event {
                    self.update_cursor_ssp(curpos_ssp);
                }

                match (&mut self.state, event) {
                    // drag/area select - todo move to viewport - content should allow viewport to discern areaselect or drag
                    (
                        SchematicSt::Idle,
                        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
                    ) => {
                        let mut click_selected = false;

                        for s in &self.selected {
                            if s.contains_ssp(curpos_ssp) {
                                click_selected = true;
                                break;
                            }
                        }

                        if click_selected {
                            self.state = SchematicSt::Moving(Some((
                                curpos_ssp,
                                curpos_ssp,
                                SSTransform::identity(),
                            )));
                        } else {
                            self.state =
                                SchematicSt::AreaSelect(SSBox::new(curpos_ssp, curpos_ssp));
                        }
                    }

                    // area select
                    (
                        SchematicSt::AreaSelect(_),
                        Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
                    ) => {
                        self.tentatives_to_selected();
                        self.state = SchematicSt::Idle;
                        clear_passive = true;
                    }
                    // moving
                    (
                        _,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::M,
                            modifiers: _,
                        }),
                    ) => {
                        self.state = SchematicSt::Moving(None);
                    }
                    (
                        SchematicSt::Moving(Some((_ssp0, _ssp1, sst))),
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::R,
                            modifiers: _,
                        }),
                    ) => {
                        *sst = sst.then(&transforms::SST_CWR);
                    }
                    (
                        SchematicSt::Moving(mut opt_pts),
                        Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
                    ) => {
                        if let Some((ssp0, ssp1, sst)) = &mut opt_pts {
                            self.content.move_elements(&self.selected, &sst.then_translate(*ssp1 - *ssp0));
                            clear_passive = true;
                            self.state = SchematicSt::Idle;
                        } else {
                            let sst = SSTransform::identity();
                            self.state = SchematicSt::Moving(Some((curpos_ssp, curpos_ssp, sst)));
                        }
                    }
                    // copying
                    (
                        SchematicSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::C,
                            modifiers: Modifiers::CTRL,
                        }),
                    ) => {
                        self.state = SchematicSt::Copying(Some((
                            curpos_ssp,
                            curpos_ssp,
                            SSTransform::identity(),
                        )));
                    }
                    (
                        SchematicSt::Copying(mut opt_pts),
                        Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
                    ) => {
                        if let Some((ssp0, ssp1, sst)) = &mut opt_pts {
                            self.content.copy_elements(&self.selected, sst);
                            clear_passive = true;
                            self.state = SchematicSt::Idle;
                        }
                    }
                    // delete
                    (
                        SchematicSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Delete,
                            modifiers: _,
                        }),
                    ) => {
                        self.content.delete_elements(&self.selected);
                        self.selected.clear();
                        clear_passive = true;
                    }
                    // tentative selection cycle
                    (
                        SchematicSt::Idle,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::C,
                            modifiers: _,
                        }),
                    ) => {
                        self.tentative_next_by_ssp(curpos_ssp);
                        clear_passive = true;
                    },

                    // rst
                    (
                        st,
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Escape,
                            modifiers: _,
                        }),
                    ) => match st {
                        SchematicSt::Idle => {
                            self.selected.clear();
                            clear_passive = true;
                        }
                        _ => {
                            self.state = SchematicSt::Idle;
                        }
                    },
                    // something else
                    _ => {}
                }
                let m = self.content.update(M::canvas_event_msg(event, curpos_ssp));
                clear_passive = clear_passive || self.update(Msg::SchematicMsg(m));
            }
            Msg::ContentMsg(content_msg) => {
                let m = self.content.update(content_msg);
                clear_passive = self.update(Msg::SchematicMsg(m));
            }
            Msg::SchematicMsg(schematic_msg) => {
                match schematic_msg {
                    SchematicMsg::None => {}
                    SchematicMsg::NewElement(e) => {
                        // place into selected
                        self.selected.clear();
                        self.selected.insert(e.take());
                        self.state = SchematicSt::Moving(Some((
                            SSPoint::origin(),
                            self.curpos_ssp,
                            SSTransform::identity(),
                        )));
                    }
                }
            }
        }
        clear_passive
    }
}

impl<C, T, M> Schematic<C, T, M>
where
    C: Content<T, M>,
    T: SchematicElement,
{
    /// returns `Some<RcRDevice>` if there is exactly 1 device in selected, otherwise returns none
    pub fn active_device(&self) -> Option<&T> {
        let mut v: Vec<_> = self.selected.iter().collect();
        if v.len() == 1 {
            v.pop()
        } else {
            None
        }
    }
    /// update schematic cursor position
    fn update_cursor_ssp(&mut self, curpos_ssp: SSPoint) {
        self.curpos_ssp = curpos_ssp;
        self.tentative_by_sspoint(curpos_ssp, &mut 0);
        let mut skip = self.selskip;

        let mut stcp = self.state.clone();
        match &mut stcp {
            SchematicSt::AreaSelect(ssb) => {
                ssb.max = curpos_ssp;
                self.tentatives_by_ssbox(ssb);
            }
            SchematicSt::Moving(Some((_ssp0, ssp1, _sst))) => {
                *ssp1 = curpos_ssp;
            }
            _ => {}
        }
        self.state = stcp;
    }
    /// set tentative flags by intersection with ssb
    pub fn tentatives_by_ssbox(&mut self, ssb: &SSBox) {
        self.content.clear_tentatives();
        let ssb_p = SSBox::from_points([ssb.min, ssb.max]).inflate(1, 1);
        self.content.tentatives_by_ssbox(ssb_p);
    }
    /// set 1 tentative flag by ssp, skipping skip elements which contains ssp. Returns netname if tentative is a net segment
    pub fn tentative_by_sspoint(&mut self, ssp: SSPoint, skip: &mut usize) {
        self.content.clear_tentatives();
        if let Some(mut e) = self.selectable(ssp, skip) {
            self.content.set_tentative(e);
            // e.set_tentative();
        }
    }
    /// set 1 tentative flag by ssp, sets flag on next qualifying element. Returns netname i tentative is a net segment
    pub fn tentative_next_by_ssp(&mut self, ssp: SSPoint) {
        let mut skip = self.selskip.wrapping_add(1);
        self.tentative_by_sspoint(ssp, &mut skip);
        self.selskip = skip;
    }
    /// put every element with tentative flag set into selected vector
    fn tentatives_to_selected(&mut self) {
        let _: Vec<_> = self
            .content
            .tentatives()
            .iter()
            .map(|e| {
                self.selected.insert(e.clone());
            })
            .collect();
    }
    /// returns true if ssp is occupied by an element
    fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.content.occupies_ssp(ssp)
    }
    /// set 1 tentative flag based on ssp and skip number. Returns the flagged element, if any.
    fn selectable(&mut self, ssp: SSPoint, skip: &mut usize) -> Option<T> {
        loop {
            let mut count = 0; // tracks the number of skipped elements
            if let Some(e) = self.content.selectable(ssp, skip, &mut count) {
                return Some(e);
            }
            if count == 0 {
                *skip = count;
                return None;
            }
            *skip -= count;
        }
    }
    /// delete all elements which appear in the selected array
    pub fn delete_selected(&mut self) {
        if let SchematicSt::Idle = self.state {
            self.content.delete(&self.selected);
            self.selected.clear();
        }
    }
    /// move all elements in the selected array by sst
    fn transform_selected(&mut self, sst: SSTransform) {
        self.content.transform(&self.selected);
        self.selected.clear();
    }
}
