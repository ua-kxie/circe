//! Schematic
//! Space in which devices and nets live in

mod devices;
pub(crate) mod nets;

use self::devices::Devices;
pub use self::devices::RcRDevice;
use crate::{interactable, viewport};
use crate::{
    interactable::Interactive,
    transforms::{
        self, CSPoint, Point, SSBox, SSPoint, SSTransform, SSVec, VCTransform, VSBox, ViewportSpace,
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
use std::hash::Hash;
use std::{collections::HashSet, fs};

// /// trait for a type of element in schematic. e.g. nets or devices
// pub trait SchematicSet {
//     /// returns the first element after skip which intersects with curpos_ssp in a BaseElement, if any.
//     /// count is incremented by 1 for every element skipped over
//     /// skip is updated if an element is returned, equal to count
//     fn selectable(
//         &mut self,
//         curpos_ssp: SSPoint,
//         skip: &mut usize,
//         count: &mut usize,
//     ) -> Option<BaseElement>;

//     /// returns the bounding box of all contained elements
//     fn bounding_box(&self) -> VSBox;
// }

// /// an enum to unify different types in schematic (nets and devices)
// #[derive(Debug, Clone)]
// pub enum BaseElement {
//     NetEdge(NetEdge),
//     Device(RcRDevice),
// }

// impl PartialEq for BaseElement {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Self::NetEdge(l0), Self::NetEdge(r0)) => *l0 == *r0,
//             (Self::Device(l0), Self::Device(r0)) => {
//                 by_address::ByAddress(l0) == by_address::ByAddress(r0)
//             }
//             _ => false,
//         }
//     }
// }

// impl Eq for BaseElement {}

// impl std::hash::Hash for BaseElement {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         match self {
//             BaseElement::NetEdge(e) => e.hash(state),
//             BaseElement::Device(d) => by_address::ByAddress(d).hash(state),
//         }
//     }
// }

pub trait SchematicElement: Hash + Eq + Drawable {
    // device designer: line, arc
    // circuit: wire, device
}

#[derive(Debug, Clone, Copy)]
pub enum SchematicMsg {
    Event(Event),
    NewState(SchematicSt),
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SchematicSt {
    #[default]
    Idle,
    AreaSelect(SSBox),
    TransformSelected(Option<(SSPoint, SSPoint, SSTransform)>),
    // first click, second click, transform for rotation/flip ONLY
}

impl SchematicSt {
    fn move_transform(ssp0: &SSPoint, ssp1: &SSPoint, sst: &SSTransform) -> SSTransform {
        sst.pre_translate(SSVec::new(-ssp0.x, -ssp0.y))
            .then_translate(SSVec::new(ssp0.x, ssp0.y))
            .then_translate(*ssp1 - *ssp0)
    }
}

pub trait Content: Drawable {
    // ex. wires + devices
    fn bounds() -> VSBox;
}

/// struct holding schematic state (nets, devices, and their locations)
#[derive(Debug, Default, Clone)]
pub struct Schematic<C, T>
where
    C: Content,
    T: SchematicElement,
{
    state: SchematicSt,
    content: C,
    selskip: usize,
    selected: HashSet<T>,
}

impl<C, T> viewport::Content<SchematicMsg> for Schematic<C, T> {
    fn mouse_interaction(&self) -> mouse::Interaction {
        match self.state {
            SchematicSt::Idle => mouse::Interaction::default(),
            SchematicSt::TransformSelected(_) => mouse::Interaction::Grabbing,
            SchematicSt::AreaSelect(_) => mouse::Interaction::Crosshair,
        }
    }

    fn events_handler(&self, event: Event) -> Option<SchematicMsg> {
        match (&self.state, event) {
            // drag/area select
            (
                SchematicSt::Idle,
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
            ) => {
                let mut click_selected = false;
                let curpos_ssp = self.curpos_ssp();

                for s in &self.selected {
                    if s.contains_ssp(curpos_ssp) {
                        click_selected = true;
                        break;
                    }
                }

                if click_selected {
                    Some(SchematicMsg::NewState(SchematicSt::TransformSelected(
                        Some((curpos_ssp, curpos_ssp, SSTransform::identity())),
                    )))
                } else {
                    Some(SchematicMsg::NewState(SchematicSt::AreaSelect(SSBox::new(
                        curpos_ssp, curpos_ssp,
                    ))))
                }
            }
            (_, _) => Some(SchematicMsg::Event(event)),
        }
    }

    /// draw onto active cache
    fn draw_active(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.content.draw_preview();

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
            SchematicSt::TransformSelected(Some((ssp0, ssp1, sst))) => {
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
    }
    /// draw onto passive cache
    fn draw_passive(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        self.content.draw_persistent();
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
    fn update(&mut self, msg: SchematicMsg, curpos_ssp: SSPoint) -> bool {
        let mut clear_passive = false;
        match msg {
            SchematicMsg::TransformInit => {
                self.state = SchematicSt::TransformSelected(None);
            }
            SchematicMsg::Event(event) => {
                if let Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = event {
                    self.update_cursor_ssp(curpos_ssp);
                }

                let mut state = self.state.clone();
                match (&mut state, event) {
                    // drag/area select
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
                            state = SchematicSt::TransformSelected(Some((
                                curpos_ssp,
                                curpos_ssp,
                                SSTransform::identity(),
                            )));
                        } else {
                            state = SchematicSt::AreaSelect(SSBox::new(curpos_ssp, curpos_ssp));
                        }
                    }

                    // area select
                    (
                        SchematicSt::AreaSelect(_),
                        Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
                    ) => {
                        self.tentatives_to_selected();
                        state = SchematicSt::Idle;
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
                        state = SchematicSt::TransformSelected(None);
                    }
                    (
                        SchematicSt::TransformSelected(Some((_ssp0, _ssp1, sst))),
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::R,
                            modifiers: modifier,
                        }),
                    ) => {
                        if modifier.shift() {
                            *sst = sst.then(&transforms::SST_CCWR);
                        } else {
                            *sst = sst.then(&transforms::SST_CWR);
                        }
                    }
                    (
                        SchematicSt::TransformSelected(mut opt_pts),
                        Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
                    ) => {
                        if let Some((ssp0, ssp1, vvt)) = &mut opt_pts {
                            // end transform if already started
                            self.move_selected(SchematicSt::move_transform(ssp0, ssp1, vvt));
                            self.prune_nets();
                            state = SchematicSt::Idle;
                            clear_passive = true;
                        } else {
                            // start transform if not yet started
                            let ssp: euclid::Point2D<_, _> = curpos_ssp;
                            let sst = SSTransform::identity();
                            state = SchematicSt::TransformSelected(Some((ssp, ssp, sst)));
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
                        self.delete_selected();
                        clear_passive = true;
                    }
                    _ => {}
                }
                self.state = state;
            }
        }
        clear_passive
    }
    fn rst(&mut self) -> bool {
        // esc - esc content first, then schematic
        match self.state {
            SchematicSt::Idle => {
                self.selected.clear();
                true
            }
            _ => {
                self.state = SchematicSt::Idle;
                false
            }
        }
    }
    fn cycle(&mut self, curpos_ssp: SSPoint) -> bool {
        if let SchematicSt::Idle = self.state {
            self.infobarstr = self.tentative_next_by_ssp(curpos_ssp);
            true
        } else {
            false
        }
    }
}

impl<C, T> Schematic<C, T> {
    /// update schematic cursor position
    fn update_cursor_ssp(&mut self, curpos_ssp: SSPoint) {
        let mut skip = self.selskip;
        self.content.update_cursor_ssp(curpos_ssp);

        let mut stcp = self.state.clone();
        match &mut stcp {
            SchematicSt::AreaSelect(ssb) => {
                ssb.max = curpos_ssp;
                self.tentatives_by_ssbox(ssb);
            }
            SchematicSt::TransformSelected(Some((_ssp0, ssp1, _sst))) => {
                *ssp1 = curpos_ssp;
            }
            _ => {}
        }
        self.state = stcp;
    }
    /// clear tentative selections (cursor hover highlight)
    fn clear_tentatives(&mut self) {
        self.devices.clear_tentatives();
        self.nets.clear_tentatives();
    }
    /// set tentative flags by intersection with ssb
    pub fn tentatives_by_ssbox(&mut self, ssb: &SSBox) {
        self.clear_tentatives();
        let ssb_p = SSBox::from_points([ssb.min, ssb.max]).inflate(1, 1);
        self.devices.tentatives_by_ssbox(&ssb_p);
        self.nets.tentatives_by_ssbox(&ssb_p);
    }
    /// set 1 tentative flag by ssp, skipping skip elements which contains ssp. Returns netname if tentative is a net segment
    pub fn tentative_by_sspoint(&mut self, ssp: SSPoint, skip: &mut usize) {
        self.clear_tentatives();
        if let Some(e) = self.selectable(ssp, skip) {
            e.set_tentative();
        }
    }
    /// set 1 tentative flag by ssp, sets flag on next qualifying element. Returns netname i tentative is a net segment
    pub fn tentative_next_by_ssp(&mut self, ssp: SSPoint) -> Option<String> {
        let mut skip = self.selskip.wrapping_add(1);
        let s = self.tentative_by_sspoint(ssp, &mut skip);
        self.selskip = skip;
        s
    }
    /// put every element with tentative flag set into selected vector
    fn tentatives_to_selected(&mut self) {
        let _: Vec<_> = self
            .content
            .tentatives()
            .map(|e| {
                self.selected.insert(e);
            })
            .collect();
    }
    /// returns true if ssp is occupied by an element
    fn occupies_ssp(&self, ssp: SSPoint) -> bool {
        self.nets.occupies_ssp(ssp) || self.devices.occupies_ssp(ssp)
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
            for e in &self.selected {
                e.delete();
            }
            self.selected.clear();
            self.prune_nets();
        }
    }
    /// move all elements in the selected array by sst
    fn move_selected(&mut self, sst: SSTransform) {
        let selected = self.selected.clone();
        self.selected.clear();
        for e in selected {
            e.transform(sst);
        }
    }
}
