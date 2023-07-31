//! Schematic
//! Space in which devices and nets live in

pub(crate) mod devices;
pub(crate) mod nets;

pub use self::devices::RcRDevice;
use crate::transforms::CSVec;
use crate::{interactable, viewport};
use crate::{
    transforms::{
        self, CSPoint, Point, SSBox, SSPoint, SSTransform, VCTransform, VSBox, VSPoint,
        ViewportSpace,
    },
    viewport::Drawable,
};
use iced::keyboard::Modifiers;
use iced::widget::canvas::{stroke, Path};
use iced::{
    mouse,
    widget::canvas::{self, event::Event, path::Builder, Frame, LineCap, Stroke},
    Color, Size,
};
use nets::Nets;
use send_wrapper::SendWrapper;
use std::collections::HashSet;
use std::hash::Hash;

pub trait SchematicElement: Hash + Eq + Drawable + Clone {
    /// returns true if self contains ssp
    fn contains_ssp(&self, ssp: SSPoint) -> bool;
}

/// Internal Schematic Message
#[derive(Debug, Clone)]
pub enum SchematicMsg<E>
where
    E: SchematicElement,
{
    /// do nothing
    None,
    /// clear passive cache
    ClearPassive,
    /// place new element E
    NewElement(SendWrapper<E>),
}

/// Trait for message type of schematic content
pub trait ContentMsg {
    /// Create message to have schematic content process canvas event
    fn canvas_event_msg(event: Event, curpos_ssp: SSPoint) -> Self;
}

/// Message type which is a composite of canvas Event, SchematicMsg, and ContentMsg
/// This structure allows schematic and its content to process events in parallel
#[derive(Debug, Clone)]
pub enum Msg<M, E>
where
    M: ContentMsg,
    E: SchematicElement,
{
    /// iced canvas event, along with cursor position inside canvas bounds
    Event(Event, Option<VSPoint>),
    /// Schematic Message
    SchematicMsg(SchematicMsg<E>),
    /// Content Message
    ContentMsg(M),
}

/// implements Msg to be ContentMsg of viewport
impl<M, E> viewport::ContentMsg for Msg<M, E>
where
    M: ContentMsg,
    E: SchematicElement,
{
    // create event to handle iced canvas event
    fn canvas_event_msg(event: Event, curpos_vsp: Option<VSPoint>) -> Self {
        Msg::Event(event, curpos_vsp.map(|vsp| vsp.round().cast().cast_unit()))
    }
}

/// Schematic States
#[derive(Debug, Clone, Copy, Default)]
pub enum SchematicSt {
    /// idle state
    #[default]
    Idle,
    /// left click-drag area selection
    AreaSelect(SSBox),
    /// selected elements preview follow mouse cursor - move, new device,
    Moving(Option<(SSPoint, SSPoint, SSTransform)>),
    /// identical to `Moving` state but signals content to make copy of elements instead of move
    Copying(Option<(SSPoint, SSPoint, SSTransform)>),
}

impl SchematicSt {
    /// this function returns a transform which applies sst about ssp0 and then translates to ssp1
    fn move_transform(ssp0: &SSPoint, ssp1: &SSPoint, sst: &SSTransform) -> SSTransform {
        sst.pre_translate(SSPoint::origin() - *ssp0)
            .then_translate(*ssp0 - SSPoint::origin())
            .then_translate(*ssp1 - *ssp0)
    }
}

pub trait Content<E, M>: Drawable + Default
where
    E: SchematicElement,
{
    /// return true if content is in its default/idle state
    fn is_idle(&self) -> bool;
    /// apply sst to elements
    fn move_elements(&mut self, elements: &HashSet<E>, sst: &SSTransform);
    /// apply sst to a copy of elements
    fn copy_elements(&mut self, elements: &HashSet<E>, sst: &SSTransform);
    /// delete elements
    fn delete_elements(&mut self, elements: &HashSet<E>);
    /// process message, returns whether or not to clear the passive cache
    fn update(&mut self, msg: M) -> SchematicMsg<E>;
    /// return bounds which enclose all elements
    fn bounds(&self) -> VSBox;
    /// return whether or not ssp intersects with any schematic element
    fn occupies_ssp(&self, ssp: SSPoint) -> bool;
    /// returns a single SchematicElement over which ssp lies. Skips the first skip elements
    fn selectable(&mut self, ssp: SSPoint, skip: usize, count: &mut usize) -> Option<E>;
    ///  returns hashset of elements which intersects ssb
    fn intersects_ssb(&mut self, ssb: SSBox) -> HashSet<E>;
}

/// struct holding schematic state (nets, devices, and their locations)
#[derive(Debug, Clone)]
pub struct Schematic<C, E, M>
where
    C: Content<E, M>,
    E: SchematicElement,
{
    /// active element
    pub active_element: Option<E>,
    /// schematic state
    state: SchematicSt,
    /// schematic content - circuit or device designer
    pub content: C,
    /// phantom data to mark ContentMsg type
    content_msg: std::marker::PhantomData<M>,
    /// single selection cycling watermark
    selskip: usize,
    /// Hashset of selected elements
    selected: HashSet<E>,
    /// Hashset of tentative elements (mouse hovering over, inside area selection)
    tentatives: HashSet<E>,
    /// cursor position in schematic space
    curpos_ssp: SSPoint,
}

impl<C, E, M> Default for Schematic<C, E, M>
where
    C: Content<E, M>,
    E: SchematicElement,
{
    fn default() -> Self {
        Self {
            active_element: None,
            state: Default::default(),
            content: Default::default(),
            selskip: Default::default(),
            selected: Default::default(),
            tentatives: Default::default(),
            content_msg: std::marker::PhantomData,
            curpos_ssp: Default::default(),
        }
    }
}

/// implement Schematic as viewport content
impl<C, E, M> viewport::Content<Msg<M, E>> for Schematic<C, E, M>
where
    M: ContentMsg,
    C: Content<E, M>,
    E: SchematicElement,
{
    /// change cursor graphic based on schematic state
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
            SchematicSt::Moving(Some((ssp0, ssp1, sst)))
            | SchematicSt::Copying(Some((ssp0, ssp1, sst))) => {
                // draw selected preview with transform applied
                let vvt = transforms::sst_to_xxt::<ViewportSpace>(SchematicSt::move_transform(
                    ssp0, ssp1, sst,
                ));

                let vct_c = vvt.then(&vct);
                for be in &self.selected {
                    be.draw_preview(vct_c, vcscale, frame);
                }
            }
            _ => {}
        }

        // draw preview for tentatives
        let _: Vec<_> = self
            .tentatives
            .iter()
            .map(|e| e.draw_preview(vct, vcscale, frame))
            .collect();
        // draw content preview
        self.content.draw_preview(vct, vcscale, frame);

        /// draw the cursor onto canvas
        pub fn draw_cursor(vct: VCTransform, frame: &mut Frame, curpos_ssp: SSPoint) {
            let cursor_stroke = || -> Stroke {
                Stroke {
                    width: 1.0,
                    style: stroke::Style::Solid(Color::from_rgb(1.0, 0.9, 0.0)),
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                }
            };
            let curdim = 5.0;
            let csp = vct.transform_point(curpos_ssp.cast().cast_unit());
            let csp_topleft = csp - CSVec::from([curdim / 2.; 2]);
            let s = iced::Size::from([curdim, curdim]);
            let c = Path::rectangle(iced::Point::from([csp_topleft.x, csp_topleft.y]), s);
            frame.stroke(&c, cursor_stroke());
        }
        draw_cursor(vct, frame, self.curpos_ssp);
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

        match msg {
            Msg::Event(event, curpos_vsp) => {
                if curpos_vsp.is_none() {
                    return false;
                }
                let curpos_ssp = curpos_vsp.unwrap().round().cast().cast_unit();

                if let Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = event {
                    self.update_cursor_ssp(curpos_ssp);
                }

                if self.content.is_idle() {
                    // if content is idle, allow schematic to process event before passing onto content - otherwise pass event to content directly
                    match (&mut self.state, event) {
                        // drag/area select - todo move to viewport - content should allow viewport to discern areaselect or drag
                        (
                            SchematicSt::Idle,
                            Event::Mouse(iced::mouse::Event::ButtonPressed(
                                iced::mouse::Button::Left,
                            )),
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
                            Event::Mouse(iced::mouse::Event::ButtonReleased(
                                iced::mouse::Button::Left,
                            )),
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
                                modifiers: m,
                            }),
                        ) => {
                            if m.control() {
                                *sst = sst.then(&transforms::SST_CCWR);
                            } else {
                                *sst = sst.then(&transforms::SST_CWR);
                            }
                        }
                        (
                            SchematicSt::Moving(mut opt_pts),
                            Event::Mouse(iced::mouse::Event::ButtonReleased(
                                iced::mouse::Button::Left,
                            )),
                        ) => {
                            if let Some((ssp0, ssp1, sst)) = &mut opt_pts {
                                self.content.move_elements(
                                    &self.selected,
                                    &SchematicSt::move_transform(ssp0, ssp1, sst),
                                );
                                clear_passive = true;
                                self.state = SchematicSt::Idle;
                            } else {
                                let sst = SSTransform::identity();
                                self.state =
                                    SchematicSt::Moving(Some((curpos_ssp, curpos_ssp, sst)));
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
                            self.state = SchematicSt::Copying(None);
                        }
                        (
                            SchematicSt::Copying(opt_pts),
                            Event::Mouse(iced::mouse::Event::ButtonReleased(
                                iced::mouse::Button::Left,
                            )),
                        ) => match opt_pts {
                            Some((ssp0, ssp1, sst)) => {
                                self.content.copy_elements(
                                    &self.selected,
                                    &SchematicSt::move_transform(ssp0, ssp1, sst),
                                );
                                clear_passive = true;
                                self.state = SchematicSt::Idle;
                            }
                            None => {
                                self.state = SchematicSt::Copying(Some((
                                    curpos_ssp,
                                    curpos_ssp,
                                    SSTransform::identity(),
                                )));
                            }
                        },
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
                        }

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
                        // something else - pass to content
                        _ => {
                            let m = self.content.update(M::canvas_event_msg(event, curpos_ssp));
                            clear_passive = self.update(Msg::SchematicMsg(m));
                        }
                    }
                } else {
                    // if content is not idling, pass event directly to content
                    let m = self.content.update(M::canvas_event_msg(event, curpos_ssp));
                    clear_passive = self.update(Msg::SchematicMsg(m));
                }
            }
            Msg::ContentMsg(content_msg) => {
                let m = self.content.update(content_msg);
                clear_passive = self.update(Msg::SchematicMsg(m));
            }
            Msg::SchematicMsg(schematic_msg) => {
                match schematic_msg {
                    SchematicMsg::None => {}
                    SchematicMsg::ClearPassive => {
                        clear_passive = true;
                    }
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

impl<C, E, M> Schematic<C, E, M>
where
    C: Content<E, M>,
    E: SchematicElement,
{
    // /// returns `Some<E>` if there is exactly 1 element in selected, otherwise returns none
    // pub fn active_element(&self) -> Option<&E> {
    //     let mut v: Vec<_> = self.selected.iter().collect();
    //     if v.len() == 1 {
    //         v.pop()
    //     } else {
    //         None
    //     }
    // }
    /// update schematic cursor position
    fn update_cursor_ssp(&mut self, curpos_ssp: SSPoint) {
        self.curpos_ssp = curpos_ssp;
        self.tentative_by_sspoint(curpos_ssp, &mut self.selskip.clone());

        let mut stcp = self.state;
        match &mut stcp {
            SchematicSt::AreaSelect(ssb) => {
                ssb.max = curpos_ssp;
                self.tentatives_by_ssbox(ssb);
            }
            SchematicSt::Moving(Some((_ssp0, ssp1, _sst))) => {
                *ssp1 = curpos_ssp;
            }
            SchematicSt::Copying(Some((_ssp0, ssp1, _sst))) => {
                *ssp1 = curpos_ssp;
            }
            _ => {}
        }
        self.state = stcp;
    }
    /// set tentative flags by intersection with ssb
    pub fn tentatives_by_ssbox(&mut self, ssb: &SSBox) {
        let ssb_p = SSBox::from_points([ssb.min, ssb.max]).inflate(1, 1);
        self.tentatives = self.content.intersects_ssb(ssb_p)
    }
    /// set 1 tentative flag by ssp, skipping skip elements which contains ssp. Returns netname if tentative is a net segment
    pub fn tentative_by_sspoint(&mut self, ssp: SSPoint, skip: &mut usize) {
        self.tentatives.clear();
        if let Some(e) = self.selectable(ssp, skip) {
            self.tentatives.insert(e);
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
        self.selected = self.tentatives.clone();
        if self.tentatives.len() == 1 {
            let mut v: Vec<_> = self.tentatives.iter().collect();
            self.active_element = v.pop().cloned();
        }
        self.tentatives.clear();
    }
    /// set 1 tentative flag based on ssp and skip number. Returns the flagged element, if any.
    fn selectable(&mut self, ssp: SSPoint, skip: &mut usize) -> Option<E> {
        loop {
            let mut count = 0; // tracks the number of skipped elements
            if let Some(e) = self.content.selectable(ssp, *skip, &mut count) {
                return Some(e);
            }
            if count == 0 {
                *skip = 0;
                return None;
            }
            *skip -= count;
        }
    }
}
