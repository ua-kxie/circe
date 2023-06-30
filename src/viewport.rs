//! the viewport handles visual transforms from the schematic to canvas and vice-versa
//! CanvasSpace <-> ViewportSpace <-> SchematicSpace
//! CanvasSpace is the UI canvas coordinate
//! ViewportSpace is the schematic coordinate in f32
//! SchematicSpace is the schematic coordinate in i16
//! separated from schematic controls - wouldn't want panning or zooming to cancel placing a device, etc.

use crate::transforms::{
    CSBox, CSPoint, CSVec, CVTransform, Point, SSPoint, VCTransform, VSBox, VSPoint, VSVec,
};
use iced::widget::canvas::path::Builder;
use iced::widget::canvas::{stroke, Event, Frame, LineCap, LineDash, Path, Stroke, Text, Cache};
use iced::Color;

/// trait for element which can be drawn on canvas
pub trait Drawable {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
}

#[derive(Clone, Debug)]
pub enum ViewportState {
    Panning(CSPoint),
    NewView(VSPoint, VSPoint),
    None,
}

impl Default for ViewportState {
    fn default() -> Self {
        ViewportState::None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ViewportMsg {
    NewView(VCTransform, f32, CSPoint),
    CursorMoved(CSPoint),
}

pub struct Viewport {
    /// iced canvas graphical cache, cleared every frame
    pub active_cache: Cache,
    /// iced canvas graphical cache, cleared following some schematic actions
    pub passive_cache: Cache,
    /// iced canvas graphical cache, almost never cleared
    pub background_cache: Cache,

    vct: VCTransform,
    zoom_scale: f32,

    curpos: (CSPoint, VSPoint, SSPoint),

    /// zoom in limit
    max_zoom: f32,
    /// zoom out limit
    min_zoom: f32,
    /// ssp always rounds to i16. This scale allows snapping to fixed f32 intervals if not 1.0
    /// effectively the transform from schematic space to viewport space
    scale: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            active_cache: Default::default(),
            passive_cache: Default::default(),
            background_cache: Default::default(),
            // state: Default::default(),
            vct: VCTransform::default()
                .pre_scale(10., 10.)
                .then_scale(1., -1.),
            zoom_scale: 10.0, // scale from canvas to viewport, sqrt of transform determinant. Save value to save computing power

            curpos: (CSPoint::origin(), VSPoint::origin(), SSPoint::origin()),

            /// most zoomed in - every 1.0 unit is 1000.0 pixels
            max_zoom: 100.0,
            /// most zoomed out - every 1.0 unit is 1.0 pixels
            min_zoom: 1.0,

            /// schematic, designer should always snap to nearest integer.  
            /// snap scale just scales viewport grid such that snapping appears to operate on some other granularity
            scale: 1.0,
        }
    }
}

impl Viewport {
    pub fn new(scale: f32, min_zoom: f32, max_zoom: f32, zoom_scale: f32, vct: VCTransform) -> Self {
        Viewport { 
            scale,
            min_zoom,
            max_zoom,
            zoom_scale,
            vct,
            ..Default::default()
        }
    }
    pub fn update(&mut self, viewport_msg: ViewportMsg) {
        match viewport_msg {
            ViewportMsg::NewView(cvt, zoom_scale, curpos_csp) => {
                self.vct = cvt;
                self.zoom_scale = zoom_scale;
                // update cursor position, otherwise it may be wrong until cursor is moved again
                self.curpos_update(curpos_csp);
            }
            ViewportMsg::CursorMoved(csp) => {
                self.curpos_update(csp);
            }
        }
    }
    /// marked for rework
    /// mutate viewport based on event
    pub fn events_handler(
        &self,
        state: &mut ViewportState,
        event: iced::widget::canvas::Event,
        bounds_csb: CSBox,
        curpos_csp: CSPoint,
    ) -> Option<ViewportMsg> {
        let mut msg = None;
        let mut stcp = state.clone();
        match (&mut stcp, event) {
            // cursor move
            (ViewportState::None, Event::Mouse(iced::mouse::Event::CursorMoved { .. })) => {
                msg = Some(ViewportMsg::CursorMoved(curpos_csp));
            }
            // zooming
            (_, Event::Mouse(iced::mouse::Event::WheelScrolled { delta })) => match delta {
                iced::mouse::ScrollDelta::Lines { y, .. }
                | iced::mouse::ScrollDelta::Pixels { y, .. } => {
                    let zoom_scale = 1.0 + y.clamp(-5.0, 5.0) / 5.;
                    msg = Some(self.zoom(zoom_scale, curpos_csp));
                }
            },
            // panning
            (
                ViewportState::None,
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Middle)),
            ) => {
                stcp = ViewportState::Panning(curpos_csp);
            }
            (
                ViewportState::Panning(csp_prev),
                Event::Mouse(iced::mouse::Event::CursorMoved { .. }),
            ) => {
                msg = Some(self.pan(curpos_csp, *csp_prev));
                *csp_prev = curpos_csp;
            }
            (
                ViewportState::Panning(_),
                Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Middle)),
            ) => {
                stcp = ViewportState::None;
            }
            // newview
            (
                ViewportState::None,
                Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right)),
            ) => {
                let vsp = self.cv_transform().transform_point(curpos_csp);
                stcp = ViewportState::NewView(vsp, vsp);
            }
            (
                ViewportState::NewView(vsp0, vsp1),
                Event::Mouse(iced::mouse::Event::CursorMoved { .. }),
            ) => {
                let vsp_now = self.cv_transform().transform_point(curpos_csp);
                if (vsp_now - *vsp0).length() > 10. {
                    *vsp1 = vsp_now;
                } else {
                    *vsp1 = *vsp0;
                }
                msg = Some(ViewportMsg::CursorMoved(curpos_csp));
            }
            (
                ViewportState::NewView(_vsp0, _vsp1),
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code,
                    modifiers,
                }),
            ) => {
                if let (iced::keyboard::KeyCode::Escape, 0) = (key_code, modifiers.bits()) {
                    stcp = ViewportState::None;
                }
            }
            (
                ViewportState::NewView(vsp0, vsp1),
                Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right)),
            ) => {
                if vsp1 != vsp0 {
                    msg = Some(self.display_bounds(
                        bounds_csb,
                        VSBox::from_points([vsp0, vsp1]),
                        curpos_csp,
                    ));
                }
                stcp = ViewportState::None;
            }
            _ => {}
        }
        *state = stcp;
        msg
    }

    /// returns the cursor position in canvas space
    pub fn curpos_csp(&self) -> CSPoint {
        self.curpos.0
    }

    /// returns the cursor position in viewport space
    pub fn curpos_vsp(&self) -> VSPoint {
        self.curpos.1
    }

    /// returns the cursor position in schematic space
    pub fn curpos_ssp(&self) -> SSPoint {
        self.curpos.2
    }

    /// returns the cursor position in schematic space
    pub fn curpos_vsp_scaled(&self) -> VSPoint {
        self.curpos.1 * self.scale
    }

    /// returns transform and scale such that VSBox (viewport/schematic bounds) fit inside CSBox (canvas bounds)
    fn bounds_transform(&self, csb: CSBox, vsb: VSBox) -> (VCTransform, f32) {
        let mut vct = VCTransform::identity();

        let s = (csb.height() / vsb.height())
            .min(csb.width() / vsb.width())
            .clamp(self.min_zoom, self.max_zoom); // scale from vsb to fit inside csb
        vct = vct.then_scale(s, -s);

        let v = csb.center() - vct.transform_point(vsb.center()); // vector from vsb to csb
        vct = vct.then_translate(v);

        (vct, s)
    }

    /// change transform such that VSBox (viewport/schematic bounds) fit inside CSBox (canvas bounds)
    pub fn display_bounds(&self, csb: CSBox, vsb: VSBox, csp: CSPoint) -> ViewportMsg {
        let (vct, zoom_scale) = self.bounds_transform(csb, vsb);
        ViewportMsg::NewView(vct, zoom_scale, csp)
    }

    /// pan by vector v
    pub fn pan(&self, csp_now: CSPoint, csp_prev: CSPoint) -> ViewportMsg {
        let v = self.cv_transform().transform_vector(csp_now - csp_prev);
        let vct = self.vct.pre_translate(v);
        ViewportMsg::NewView(vct, self.zoom_scale, csp_now)
    }

    /// return the canvas to viewport space transform
    pub fn cv_transform(&self) -> CVTransform {
        self.vct.inverse().unwrap()
    }

    /// return the viewport to canvas space transform
    pub fn vc_transform(&self) -> VCTransform {
        self.vct
    }

    /// returns the scale factor in the viewwport to canvas transform
    /// this value is stored to avoid calling sqrt() each time
    pub fn vc_scale(&self) -> f32 {
        self.zoom_scale
    }

    /// returns the scale factor in the viewwport to canvas transform
    /// this value is stored to avoid calling sqrt() each time
    pub fn cv_scale(&self) -> f32 {
        1. / self.zoom_scale
    }

    /// update the cursor position
    pub fn curpos_update(&mut self, csp1: CSPoint) {
        let vsp1 = self.cv_transform().transform_point(csp1);
        let ssp1: SSPoint = vsp1.round().cast().cast_unit();
        self.curpos = (csp1, vsp1, ssp1);
    }

    /// update the cursor position
    pub fn curpos(&mut self, csp1: CSPoint) -> (VSPoint, SSPoint) {
        let vsp1 = self.cv_transform().transform_point(csp1);
        let ssp1: SSPoint = vsp1.round().cast().cast_unit();
        (vsp1, ssp1)
    }

    /// change the viewport zoom by scale
    pub fn zoom(&self, zoom_scale: f32, curpos_csp: CSPoint) -> ViewportMsg {
        let (csp, vsp, _) = self.curpos;
        let scaled_transform = self.vct.then_scale(zoom_scale, zoom_scale);

        let mut new_transform; // transform with applied scale and translated to maintain p_viewport position
        let scaled_determinant = scaled_transform.determinant().abs();
        if scaled_determinant < self.min_zoom * self.min_zoom {
            // minimum scale
            let clamped_scale = self.min_zoom / self.vc_scale();
            new_transform = self.vct.then_scale(clamped_scale, clamped_scale);
        } else if scaled_determinant <= self.max_zoom * self.max_zoom {
            // adjust scale
            new_transform = scaled_transform;
        } else {
            // maximum scale
            let clamped_scale = self.max_zoom / self.vc_scale();
            new_transform = self.vct.then_scale(clamped_scale, clamped_scale);
        }
        let csp1 = new_transform.transform_point(vsp); // translate based on cursor location
        let translation = csp - csp1;
        new_transform = new_transform.then_translate(translation);

        ViewportMsg::NewView(
            new_transform,
            new_transform.determinant().abs().sqrt(),
            curpos_csp,
        )
    }

    /// draw the cursor onto canvas
    pub fn draw_cursor(&self, frame: &mut Frame) {
        let cursor_stroke = || -> Stroke {
            Stroke {
                width: 1.0,
                style: stroke::Style::Solid(Color::from_rgb(1.0, 0.9, 0.0)),
                line_cap: LineCap::Round,
                ..Stroke::default()
            }
        };
        let curdim = 5.0;
        let csp = self
            .vc_transform()
            .transform_point(self.curpos.2.cast().cast_unit());
        let csp_topleft = csp - CSVec::from([curdim / 2.; 2]);
        let s = iced::Size::from([curdim, curdim]);
        let c = Path::rectangle(iced::Point::from([csp_topleft.x, csp_topleft.y]), s);
        frame.stroke(&c, cursor_stroke());
    }

    /// draw the schematic grid onto canvas
    pub fn draw_grid(&self, frame: &mut Frame, bb_canvas: CSBox) {
        let a = Text {
            content: String::from("origin"),
            position: Point::from(self.vc_transform().transform_point(VSPoint::origin())).into(),
            color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
            size: self.vc_scale() * self.scale,
            ..Default::default()
        };
        frame.fill_text(a);

        fn draw_grid_w_spacing(
            spacing: f32,
            bb_canvas: CSBox,
            vct: VCTransform,
            cvt: CVTransform,
            frame: &mut Frame,
            stroke: Stroke,
        ) {
            let bb_viewport = cvt.outer_transformed_box(&bb_canvas);
            let v = ((bb_viewport.min / spacing).ceil() * spacing) - bb_viewport.min;
            let bb_viewport = bb_viewport.translate(v);

            let v = bb_viewport.max - bb_viewport.min;
            for col in 0..=(v.x / spacing).ceil() as u32 {
                let csp0 = bb_viewport.min + VSVec::from([col as f32 * spacing, 0.0]);
                let csp1 = bb_viewport.min + VSVec::from([col as f32 * spacing, v.y.ceil()]);
                let c = Path::line(
                    Point::from(vct.transform_point(csp0)).into(),
                    Point::from(vct.transform_point(csp1)).into(),
                );
                frame.stroke(&c, stroke.clone());
            }
        }
        let coarse_grid_threshold: f32 = 2.0 / self.scale;
        let fine_grid_threshold: f32 = 6.0 / self.scale;
        if self.vc_scale() > coarse_grid_threshold {
            // draw coarse grid
            let spacing = 16.0 * self.scale;

            let grid_stroke = Stroke {
                width: (0.5 * self.vc_scale() * self.scale).clamp(0.5, 3.0),
                style: stroke::Style::Solid(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
                line_cap: LineCap::Round,
                line_dash: LineDash {
                    segments: &[0.0, spacing * self.vc_scale()],
                    offset: 0,
                },
                ..Stroke::default()
            };
            draw_grid_w_spacing(
                spacing,
                bb_canvas,
                self.vc_transform(),
                self.cv_transform(),
                frame,
                grid_stroke,
            );

            if self.vc_scale() > fine_grid_threshold {
                // draw fine grid if sufficiently zoomed in
                let spacing = 2.0 * self.scale;

                let grid_stroke = Stroke {
                    width: 1.0,
                    style: stroke::Style::Solid(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
                    line_cap: LineCap::Round,
                    line_dash: LineDash {
                        segments: &[0.0, spacing * self.vc_scale()],
                        offset: 0,
                    },
                    ..Stroke::default()
                };

                draw_grid_w_spacing(
                    spacing,
                    bb_canvas,
                    self.vc_transform(),
                    self.cv_transform(),
                    frame,
                    grid_stroke,
                );
            }
        }
        let ref_stroke = Stroke {
            width: (0.1 * self.vc_scale() * self.scale).clamp(0.1, 3.0),
            style: stroke::Style::Solid(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };

        let mut path_builder = Builder::new();
        path_builder.move_to(
            Point::from(self.vc_transform().transform_point(VSPoint::new(0.0, 1.0) * self.scale)).into(),
        );
        path_builder.line_to(
            Point::from(self.vc_transform().transform_point(VSPoint::new(0.0, -1.0) * self.scale)).into(),
        );
        path_builder.move_to(
            Point::from(self.vc_transform().transform_point(VSPoint::new(1.0, 0.0) * self.scale)).into(),
        );
        path_builder.line_to(
            Point::from(self.vc_transform().transform_point(VSPoint::new(-1.0, 0.0) * self.scale)).into(),
        );
        let p = self.vc_transform().transform_point(VSPoint::origin());
        let r = self.vc_scale() * self.scale * 0.5;
        path_builder.circle(Point::from(p).into(), r);
        frame.stroke(&path_builder.build(), ref_stroke);
    }
}
