//! device designer
//! editor for designing devices - draw the appearance and place ports
//! intended for dev use for now, can be recycled for user use to design subcircuit (.model) devices

use crate::viewport::{self, Drawable};
use crate::IcedStruct;
use crate::{
    transforms::{
        self, CSBox, CSPoint, Point, SSBox, SSPoint, SSTransform, SSVec, VCTransform, VSBox,
        VSPoint,
    },
    viewport::{Viewport, ViewportState},
};
use iced::widget::{canvas, text};
use iced::{alignment, Length};
use iced::{
    mouse,
    widget::canvas::{
        event::{self, Event},
        Cache, Cursor, Frame, Geometry,
    },
    Color, Rectangle, Size, Theme,
};

use self::graphics::line::LineSeg;

mod graphics;

pub struct DesignerViewport(Viewport);

impl Default for DesignerViewport {
    fn default() -> Self {
        let mut v = Viewport::default();
        v.snap_scale = transforms::DESIGNER_GRID;
        DesignerViewport(v)
    }
}

#[derive(Clone)]
pub enum DesignerState {
    Idle,
    Selecting(SSBox),
    Moving(Option<(SSPoint, SSPoint, SSTransform)>),
    DrawLine(Option<(SSPoint, SSPoint)>), // first click, second click, transform for rotation/flip ONLY
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
pub struct DeviceDesigner {
    /// Viewport
    viewport: Viewport,

    /// iced canvas graphical cache, cleared every frame
    active_cache: Cache,
    /// iced canvas graphical cache, cleared following some schematic actions
    passive_cache: Cache,
    /// iced canvas graphical cache, almost never cleared
    background_cache: Cache,

    state: DesignerState,

    curpos_vsp: VSPoint,
    zoom_scale: f32,

    selskip: usize,
    selected: Vec<()>, // todo

    dev: Vec<LineSeg>,
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceDesignerMsg {
    Fit(CSBox),
    ViewportMsg(viewport::ViewportMsg),
}

impl canvas::Program<DeviceDesignerMsg> for DeviceDesigner {
    type State = ViewportState;

    fn update(
        &self,
        viewport_st: &mut ViewportState,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<DeviceDesignerMsg>) {
        let curpos = cursor.position_in(&bounds);
        let vstate = viewport_st.clone();
        let mut msg = None;
        let csb = CSBox::from_points([CSPoint::origin(), CSPoint::new(bounds.x, bounds.y)]);

        self.active_cache.clear();
        if let Some(p) = curpos {
            if let Some(msg) =
                self.viewport
                    .events_handler(viewport_st, event, csb, Point::from(p).into())
            {
                return (
                    event::Status::Captured,
                    Some(DeviceDesignerMsg::ViewportMsg(msg)),
                );
            }
        }

        // if let Some(p) = curpos {
        //     self.viewport.curpos_update(Point::from(p).into());
        //     let curpos_ssp = self.viewport.curpos_ssp();

        //     if let Event::Mouse(iced::mouse::Event::CursorMoved { .. }) = event {
        //         let mut skip = self.selskip.saturating_sub(1);
        //         self.tentative_by_vspoint(curpos_ssp, &mut skip);
        //         self.selskip = skip;
        //     }
        // }

        let mut state = self.state.clone();
        match (&mut state, event) {
            // drawing line
            // (
            //     _,
            //     Event::Keyboard(iced::keyboard::Event::KeyPressed {
            //         key_code: iced::keyboard::KeyCode::W,
            //         modifiers: _,
            //     }),
            // ) => {
            //     state = DesignerState::DrawLine(None);
            // }
            // (
            //     DesignerState::DrawLine(opt_pts),
            //     Event::Mouse(iced::mouse::Event::ButtonPressed(mouse::Button::Left)),
            // ) => match opt_pts {
            //     Some(pts) => {
            //         let l = LineSeg {
            //             src: pts.0,
            //             dst: pts.1,
            //             interactable: LineSeg::interactable(pts.0, pts.1, false),
            //         };
            //         self.dev.push(l);
            //         self.passive_cache.clear();
            //         *opt_pts = None;
            //     }
            //     None => {
            //         if let Some(p) = curpos {
            //             let (_, curpos_ssp) = self.viewport.curpos(Point::from(p).into());
            //             *opt_pts = Some((curpos_ssp, curpos_ssp));
            //         }
            //     }
            // },
            // (
            //     DesignerState::DrawLine(opt_pts),
            //     Event::Mouse(iced::mouse::Event::CursorMoved { position: _ }),
            // ) => match opt_pts {
            //     Some(pts) => {
            //         if let Some(p) = curpos {
            //             let (_, curpos_ssp) = self.viewport.curpos(Point::from(p).into());
            //             pts.1 = curpos_ssp;
            //         }
            //     }
            //     None => {}
            // },

            // // esc
            // (
            //     st,
            //     Event::Keyboard(iced::keyboard::Event::KeyPressed {
            //         key_code: iced::keyboard::KeyCode::Escape,
            //         modifiers: _,
            //     }),
            // ) => match st {
            //     DesignerState::Idle => {
            //         self.clear_selected();
            //     }
            //     _ => {
            //         state = DesignerState::Idle;
            //     }
            // },
            // // delete
            // (
            //     DesignerState::Idle,
            //     Event::Keyboard(iced::keyboard::Event::KeyPressed {
            //         key_code: iced::keyboard::KeyCode::Delete,
            //         modifiers: _,
            //     }),
            // ) => {
            //     self.delete_selected();
            // }
            // // cycle
            // (
            //     DesignerState::Idle,
            //     Event::Keyboard(iced::keyboard::Event::KeyPressed {
            //         key_code: iced::keyboard::KeyCode::C,
            //         modifiers: _,
            //     }),
            // ) => {
            //     if let Some(p) = curpos {
            //         let (_, curpos_ssp) = self.viewport.curpos(Point::from(p).into());
            //         self.tentative_next_by_vsp(curpos_ssp);
            //     }
            // }
            // fit msg
            (
                DesignerState::Idle,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::F,
                    modifiers: _,
                }),
            ) => {
                msg = Some(DeviceDesignerMsg::Fit(CSBox::from_points([
                    CSPoint::origin(),
                    CSPoint::new(bounds.x, bounds.y),
                ])));
            }
            _ => {}
        }

        if msg.is_some() {
            (event::Status::Captured, msg)
        } else {
            (event::Status::Ignored, msg)
        }
    }

    fn draw(
        &self,
        viewport_st: &ViewportState,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let active = self.active_cache.draw(bounds.size(), |frame| {
            self.draw_active(
                self.viewport.vc_transform(),
                self.viewport.vc_scale(),
                frame,
            );
            self.viewport.draw_cursor(frame);

            if let ViewportState::NewView(vsp0, vsp1) = viewport_st {
                let csp0 = self.viewport.vc_transform().transform_point(*vsp0);
                let csp1 = self.viewport.vc_transform().transform_point(*vsp1);
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
            self.viewport.draw_grid(
                frame,
                CSBox::new(
                    CSPoint::origin(),
                    CSPoint::from([bounds.width, bounds.height]),
                ),
            );
            self.draw_passive(
                self.viewport.vc_transform(),
                self.viewport.vc_scale(),
                frame,
            );
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
        viewport_st: &ViewportState,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(&bounds) {
            match (&viewport_st, &self.state) {
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

impl IcedStruct<DeviceDesignerMsg> for DeviceDesigner {
    fn update(&mut self, msg: DeviceDesignerMsg) {
        match msg {
            DeviceDesignerMsg::Fit(csb) => {
                let vsb = self.bounding_box().inflate(5.0, 5.0);
                let csp = self.viewport.curpos_csp();
                let msg = self.viewport.display_bounds(csb, vsb, csp);
                self.viewport.update(msg);
                self.passive_cache.clear();
            }
            DeviceDesignerMsg::ViewportMsg(vp_msg) => {
                self.viewport.update(vp_msg);
                self.passive_cache.clear();
            }
        }
    }

    fn view(&self) -> iced::Element<DeviceDesignerMsg> {
        let str_vsp = format!("x: {}; y: {}", self.curpos_vsp.x, self.curpos_vsp.y);

        let canvas = canvas(self).width(Length::Fill).height(Length::Fill);
        let dd = iced::widget::column![
            canvas,
            iced::widget::row![
                text(str_vsp)
                    .size(16)
                    .height(16)
                    .vertical_alignment(alignment::Vertical::Center),
                text(&format!("{:04.1}", self.zoom_scale))
                    .size(16)
                    .height(16)
                    .vertical_alignment(alignment::Vertical::Center),
            ]
            .spacing(10)
        ]
        .width(Length::Fill);
        dd.into()
    }
}

impl DeviceDesigner {
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
        match &self.state {
            DesignerState::DrawLine(Some(pts)) => {
                let l = LineSeg {
                    src: pts.0,
                    dst: pts.1,
                    interactable: LineSeg::interactable(pts.0, pts.1, false),
                };
                l.draw_preview(vct, vcscale, frame);
            }
            _ => {}
        }
    }
    /// draw onto passive cache
    pub fn draw_passive(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        // draw elements which may need to be redrawn at any event
        for l in &self.dev {
            l.draw_persistent(vct, vcscale, frame);
        }
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
}
