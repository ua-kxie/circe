//! device designer
//! editor for designing devices - draw the appearance and place ports

use crate::interactable::{Interactable, Interactive};
use crate::{
    transforms::{
        self, CSBox, CSPoint, Point, SSBox, SSPoint, SSTransform, SSVec, VCTransform, VSBox,
        VSPoint, ViewportSpace,
    },
    viewport::{Viewport, ViewportState},
    Msg,
};
use iced::{
    mouse,
    widget::canvas::{
        self,
        event::{self, Event},
        path::Builder,
        Cache, Cursor, Frame, Geometry, LineCap, Stroke,
    },
    Color, Rectangle, Size, Theme,
};
use std::{collections::HashSet, fs};

// mod graphics;  // wip

pub struct DesignerViewport(Viewport);

impl Default for DesignerViewport {
    fn default() -> Self {
        let mut v = Viewport::default();
        v.snap_scale = transforms::DESIGNER_GRID;
        DesignerViewport(
            v
        )
    }
}

/// trait for element which can be drawn on canvas
pub trait Drawable {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
}

#[derive(Clone)]
pub enum DesignerState {
    Idle,
    Selecting(SSBox),
    Moving(Option<(SSPoint, SSPoint, SSTransform)>),
    // first click, second click, transform for rotation/flip ONLY
}

impl Default for DesignerState {
    fn default() -> Self {
        DesignerState::Idle
    }
}

impl DesignerState {
    fn move_transform(ssp0: &SSPoint, ssp1: &SSPoint, sst: &SSTransform) -> SSTransform {
        sst.pre_translate(SSVec::new(-ssp0.x, -ssp0.y))
            .then_translate(SSVec::new(ssp0.x, ssp0.y))
            .then_translate(*ssp1 - *ssp0)
    }
}

/// schematic
#[derive(Default)]
pub struct Designer {
    /// iced canvas graphical cache, cleared every frame
    pub active_cache: Cache,
    /// iced canvas graphical cache, cleared following some schematic actions
    pub passive_cache: Cache,
    /// iced canvas graphical cache, almost never cleared
    pub background_cache: Cache,

    pub state: DesignerState,

    selskip: usize,
    selected: Vec<()>, // todo
}

impl canvas::Program<Msg> for Designer {
    type State = DesignerViewport;

    fn update(
        &self,
        viewport: &mut DesignerViewport,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Msg>) {
        let curpos = cursor.position_in(&bounds);
        let vstate = viewport.0.state.clone();
        let mut msg = None;

        if let Some(curpos_csp) = curpos.map(|x| Point::from(x).into()) {
            if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key_code,
                modifiers,
            }) = event
            {
                if let (_, iced::keyboard::KeyCode::F, 0, _) =
                    (vstate, key_code, modifiers.bits(), curpos)
                {
                    let vsb = self.bounding_box().inflate(5., 5.);
                    viewport.0.display_bounds(
                        CSBox::from_points([
                            CSPoint::origin(),
                            CSPoint::new(bounds.width, bounds.height),
                        ]),
                        vsb,
                    );
                    self.passive_cache.clear();
                }
            }

            let (msg0, clear_passive0, processed) =
                viewport.0.events_handler(event, curpos_csp, bounds);
            if !processed {
                msg = Some(Msg::DesignerEvent(event, viewport.0.curpos_ssp()));
            } else {
                if clear_passive0 {
                    self.passive_cache.clear()
                }
                msg = msg0;
            }

            self.active_cache.clear();
        }

        if msg.is_some() {
            (event::Status::Captured, msg)
        } else {
            (event::Status::Ignored, msg)
        }
    }

    fn draw(
        &self,
        viewport: &DesignerViewport,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let active = self.active_cache.draw(bounds.size(), |frame| {
            self.draw_active(viewport.0.vc_transform(), viewport.0.vc_scale(), frame);
            viewport.0.draw_cursor(frame);

            if let ViewportState::NewView(vsp0, vsp1) = viewport.0.state {
                let csp0 = viewport.0.vc_transform().transform_point(vsp0);
                let csp1 = viewport.0.vc_transform().transform_point(vsp1);
                let selsize = Size {
                    width: csp1.x - csp0.x,
                    height: csp1.y - csp0.y,
                };
                let f = canvas::Fill {
                    style: canvas::Style::Solid(if selsize.height > 0. {
                        Color::from_rgba(1., 0., 0., 0.1)
                    } else {
                        Color::from_rgba(0., 0., 1., 0.1)
                    }),
                    ..canvas::Fill::default()
                };
                frame.fill_rectangle(Point::from(csp0).into(), selsize, f);
            }
        });

        let passive = self.passive_cache.draw(bounds.size(), |frame| {
            viewport.0.draw_grid(
                frame,
                CSBox::new(
                    CSPoint::origin(),
                    CSPoint::from([bounds.width, bounds.height]),
                ),
            );
            self.draw_passive(viewport.0.vc_transform(), viewport.0.vc_scale(), frame);
        });

        let background = self.background_cache.draw(bounds.size(), |frame| {
            let f = canvas::Fill {
                style: canvas::Style::Solid(Color::from_rgb(0.2, 0.2, 0.2)),
                ..canvas::Fill::default()
            };
            frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), f);
        });

        vec![background, passive, active]
    }

    fn mouse_interaction(
        &self,
        viewport: &DesignerViewport,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(&bounds) {
            match (&viewport.0.state, &self.state) {
                (ViewportState::Panning(_), _) => mouse::Interaction::Grabbing,
                (ViewportState::None, DesignerState::Idle) => mouse::Interaction::default(),
                (ViewportState::None, DesignerState::Moving(_)) => {
                    mouse::Interaction::ResizingVertically
                }
                _ => mouse::Interaction::default(),
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl Designer {
    /// clear selection
    fn clear_selected(&mut self) {
        self.selected.clear();
    }
    /// clear tentative selections (cursor hover highlight)
    fn clear_tentatives(&mut self) {}
    /// set tentative flags by intersection with ssb
    pub fn tentatives_by_ssbox(&mut self, ssb: &SSBox) {
        self.clear_tentatives();
        let ssb_p = SSBox::from_points([ssb.min, ssb.max]).inflate(1, 1);
    }
    /// set 1 tentative flag by vsp, skipping skip elements which contains vsp. Returns netname if tentative is a net segment
    pub fn tentative_by_vspoint(&mut self, ssp: SSPoint, skip: &mut usize) {
        self.clear_tentatives();
        if let Some(be) = self.selectable(ssp, skip) {}
    }
    /// set 1 tentative flag by vsp, sets flag on next qualifying element. Returns netname i tentative is a net segment
    pub fn tentative_next_by_vsp(&mut self, ssp: SSPoint) {
        let mut skip = self.selskip;
        let s = self.tentative_by_vspoint(ssp, &mut skip);
        self.selskip = skip;
        s
    }
    /// put every element with tentative flag set into selected vector
    fn tentatives_to_selected(&mut self) {}
    /// draw onto active cache
    pub fn draw_active(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        // draw elements which may need to be redrawn at any event
    }
    /// draw onto passive cache
    pub fn draw_passive(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        // draw elements which may need to be redrawn at any event
    }
    /// returns the bouding box of all elements on canvas
    pub fn bounding_box(&self) -> VSBox {
        todo!()
    }
    /// set 1 tentative flag based on ssp and skip number. Returns the flagged element, if any.
    fn selectable(&mut self, ssp: SSPoint, skip: &mut usize) -> Option<()> {
        None
    }
    /// delete all elements which appear in the selected array
    pub fn delete_selected(&mut self) {}
    /// move all elements in the selected array by sst
    fn move_selected(&mut self, sst: SSTransform) {}
    /// mutate schematic based on event
    pub fn events_handler(&mut self, event: Event, curpos_ssp: SSPoint) -> bool {
        let mut clear_passive = false;

        if let Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = event {
            let mut skip = self.selskip.saturating_sub(1);
            self.tentative_by_vspoint(curpos_ssp, &mut skip);
            self.selskip = skip;
        }

        let mut state = self.state.clone();
        match (&mut state, event) {
            // esc
            (
                st,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::Escape,
                    modifiers: _,
                }),
            ) => match st {
                DesignerState::Idle => {
                    self.clear_selected();
                    clear_passive = true;
                }
                _ => {
                    state = DesignerState::Idle;
                }
            },
            // delete
            (
                DesignerState::Idle,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::Delete,
                    modifiers: _,
                }),
            ) => {
                self.delete_selected();
                clear_passive = true;
            }
            // cycle
            (
                DesignerState::Idle,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::C,
                    modifiers: _,
                }),
            ) => {
                self.tentative_next_by_vsp(curpos_ssp);
            }
            _ => {}
        }
        self.state = state;
        clear_passive
    }
}
