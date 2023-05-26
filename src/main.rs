mod transforms;
use std::fmt::Debug;

use transforms::{Point, CSPoint, VSPoint, CSBox, VSBox, SSPoint};
mod viewport;
use viewport::ViewportState;
mod schematic;
use schematic::{Schematic, SchematicState};

use iced::{executor, Size};
use iced::widget::canvas::{
    Cache, Cursor, Geometry,
};
use iced::widget::{canvas, column};
use iced::{
    Application, Color, Command, Element, Length, Rectangle, Settings,
    Theme,
};
use iced::widget::canvas::event::{self, Event};
use iced::mouse;
use euclid::{Box2D, Point2D};
use infobar::infobar;

pub fn main() -> iced::Result {
    Circe::run(Settings {
        window: iced::window::Settings {
             size: (350, 350), 
             ..iced::window::Settings::default()
            },
        antialiasing: true,
        ..Settings::default()
    })
}

struct Circe {
    zoom_scale: f32,
    curpos_ssp: Option<SSPoint>,

    active_cache: Cache,
    passive_cache: Cache,
    background_cache: Cache,
}

#[derive(Debug, Clone)]
enum Msg {
    NewCurpos(Option<SSPoint>),
    NewZoom(f32),
}

impl Application for Circe {
    type Executor = executor::Default;
    type Message = Msg;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Msg>) {
        (
            Circe {
                zoom_scale: 10.0,  // would be better to get this from the viewport on startup
                curpos_ssp: None,

                active_cache: Default::default(),
                passive_cache: Default::default(),
                background_cache: Default::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Schematic Prototyping")
    }

    fn update(&mut self, message: Msg) -> Command<Msg> {
        match message {
            Msg::NewCurpos(opt_ssp) => {
                self.curpos_ssp = opt_ssp
            }
            Msg::NewZoom(value) => {
                self.zoom_scale = value
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Msg> {
        let canvas = canvas(self as &Self)
            .width(Length::Fill)
            .height(Length::Fill);
        let infobar = infobar(self.curpos_ssp, self.zoom_scale);

        column![
            canvas,
            infobar,
        ]
        .width(Length::Fill)
        .into()
    }
}

use mouse::Event::*;
use mouse::Button::*;
use viewport::Viewport;

impl canvas::Program<Msg> for Circe {
    type State = (Viewport, Schematic);

    fn update(
        &self,
        sttup: &mut (Viewport, Schematic),
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Msg>) {
        
        let curpos = cursor.position_in(&bounds);
        let state = sttup.0.state.clone();
        let mut viewport = &mut sttup.0;
        let mut msg = None;

        // match (sttup.0.state, sttup.1.state, event) {
        //     ViewportState::None
        // }
        match (state, event, curpos) {
            // clicking
            (_, Event::Mouse(ButtonPressed(Left)), Some(_)) => {
                if let Some(vsp) = viewport.curpos_vsp() {
                    sttup.1.left_click_down(vsp);
                    self.active_cache.clear();
                    self.passive_cache.clear();
                }

            }
            (_, Event::Mouse(ButtonReleased(Left)), _) => {
                sttup.1.left_click_up();
                self.active_cache.clear();
                self.passive_cache.clear();
            }

            // panning
            (_, Event::Mouse(ButtonPressed(Middle)), Some(_)) => {
                viewport.state = ViewportState::Panning;
            }
            (ViewportState::Panning, Event::Mouse(ButtonReleased(Middle)), _) => {
                viewport.state = ViewportState::None;
            }

            // new view
            (_, Event::Mouse(ButtonPressed(Right)), Some(p)) => {
                let csp: CSPoint = Point::from(p).into();
                let vsp = viewport.cv_transform().transform_point(csp);
                viewport.state = ViewportState::NewView(vsp, vsp);
            }
            (ViewportState::NewView(vsp0, vsp1), Event::Mouse(ButtonReleased(Right)), _) => {
                if vsp1 != vsp0 {
                    viewport.display_bounds(
                        CSBox::from_points([CSPoint::origin(), CSPoint::new(bounds.width, bounds.height)]), 
                        VSBox::from_points([vsp0, vsp1])
                    );
                }
                msg = Some(Msg::NewZoom(viewport.vc_scale()));
                viewport.state = ViewportState::None;
                self.passive_cache.clear();
                self.active_cache.clear();
            }

            // zooming
            (_, Event::Mouse(WheelScrolled{delta}), Some(p)) => { match delta {
                mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => { 
                    self.active_cache.clear();
                    self.passive_cache.clear();
                    let scale = 1.0 + y.clamp(-5.0, 5.0) / 5.;
                    viewport.zoom(scale);
                }}
                msg = Some(Msg::NewZoom(viewport.vc_scale()));
            }

            // keys
            (vstate, Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code, modifiers}), curpos) => { 
                match (vstate, key_code, modifiers.bits(), curpos) {
                    (_, iced::keyboard::KeyCode::M, 0, _) => {
                        sttup.1.move_();
                    },
                    (_, iced::keyboard::KeyCode::W, 0, _) => {
                        sttup.1.enter_wiring_mode();
                    },
                    (_, iced::keyboard::KeyCode::R, 0, _) => {
                        if let Some(ssp) = viewport.curpos_ssp(){
                            sttup.1.key_r(ssp);
                            self.active_cache.clear();
                        }
                    },
                    (_, iced::keyboard::KeyCode::C, 0, _) => {
                        if let Some(vsp) = viewport.curpos_vsp(){
                            sttup.1.tentative_next_by_vspoint(vsp);
                            self.active_cache.clear();
                            self.passive_cache.clear();
                        }
                    },
                    (_, iced::keyboard::KeyCode::T, 0, _) => {
                        sttup.1.key_test();
                        self.active_cache.clear();
                        self.passive_cache.clear();
                    },
                    (_, iced::keyboard::KeyCode::F, 0, _) => {
                        let vsb = sttup.1.bounding_box().inflate(5., 5.);
                        viewport.display_bounds(
                            CSBox::from_points([CSPoint::origin(), CSPoint::new(bounds.width, bounds.height)]), 
                            vsb,
                        );
                        self.active_cache.clear();
                        self.passive_cache.clear();
                    },
                    (_, iced::keyboard::KeyCode::Escape, 0, _) => {
                        sttup.1.esc();
                        self.active_cache.clear();
                        self.passive_cache.clear();
                    },
                    (_, iced::keyboard::KeyCode::Delete, 0, _) => {
                        sttup.1.delete_selected();
                        self.active_cache.clear();
                        self.passive_cache.clear();
                    },
                    _ => {},
                }
            }

            (vstate, Event::Mouse(mouse::Event::CursorMoved { .. }), opt_csp) => {
                let opt_csp = opt_csp.map(|p| Point::from(p).into());
                self.active_cache.clear();
                if let ViewportState::Panning = vstate {
                    self.passive_cache.clear();
                }
                viewport.curpos_update(opt_csp);
                sttup.1.curpos_update(viewport.curpos_vs_ss());
                msg = Some(Msg::NewCurpos(viewport.curpos_vs_ss().map(|x| x.1)))
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
        sttup: &(Viewport, Schematic),
        _theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        let active = self.active_cache.draw(bounds.size(), |frame| {
            sttup.1.draw_active(sttup.0.vc_transform(), sttup.0.vc_scale(), frame);
            sttup.0.draw_cursor(frame);

            if let ViewportState::NewView(vsp0, vsp1) = sttup.0.state {
                let csp0 = sttup.0.vc_transform().transform_point(vsp0);
                let csp1 = sttup.0.vc_transform().transform_point(vsp1);
                let selsize = Size{width: csp1.x - csp0.x, height: csp1.y - csp0.y};
                let f = canvas::Fill {
                    style: canvas::Style::Solid(if selsize.height > 0. {Color::from_rgba(1., 0., 0., 0.1)} else {Color::from_rgba(0., 0., 1., 0.1)}),
                    ..canvas::Fill::default()
                };
                frame.fill_rectangle(Point::from(csp0).into(), selsize, f);
            }
        });

        let passive = self.passive_cache.draw(bounds.size(), |frame| {
            sttup.0.draw_grid(frame, Box2D::new(CSPoint::origin(), Point2D::from([bounds.width, bounds.height])));
            sttup.1.draw_passive(sttup.0.vc_transform(), sttup.0.vc_scale(), frame);
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
        sttup: &(Viewport, Schematic),
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(&bounds) {
            match (&sttup.0.state, &sttup.1.state) {
                (ViewportState::Panning, _) => mouse::Interaction::Grabbing,
                (ViewportState::None, SchematicState::Idle) => mouse::Interaction::default(),
                (ViewportState::None, SchematicState::Wiring(_)) => mouse::Interaction::Crosshair,
                (ViewportState::None, SchematicState::DevicePlacement(_)) => mouse::Interaction::ResizingHorizontally,
                (ViewportState::None, SchematicState::Moving(_)) => mouse::Interaction::ResizingVertically,
                _ => mouse::Interaction::default(),
            }
        } else {
            mouse::Interaction::default()
        }

    }
}

mod infobar {
    use iced::alignment::{self};
    use iced::widget::{row, text};
    use iced_lazy::{component, Component};
    use iced::{Element, Renderer};

    use crate::transforms::SSPoint;

    pub struct InfoBar {
        curpos_ssp: Option<SSPoint>,
        zoom_scale: f32,
        // net_name: Option<&'a str>,
    }
    
    impl InfoBar {
        pub fn new(
            curpos_ssp: Option<SSPoint>,
            zoom_scale: f32,
        ) -> Self {
            Self {
                curpos_ssp,
                zoom_scale,
            }
        }
    }

    pub fn infobar(
        curpos_ssp: Option<SSPoint>,
        zoom_scale: f32,
    ) -> InfoBar {
        InfoBar::new(curpos_ssp, zoom_scale)
    }

    impl<Message> Component<Message, Renderer> for InfoBar {
        type State = ();
        type Event = ();

        fn update(
            &mut self,
            _state: &mut Self::State,
            event: (),
        ) -> Option<Message> {
            None
        }
        fn view(&self, _state: &Self::State) -> Element<(), Renderer> {
            let str_ssp;
            if let Some(ssp) = self.curpos_ssp {
                str_ssp = format!("x: {}; y: {}", ssp.x, ssp.y);
            } else {
                str_ssp = String::from("");
            }

            row![
                text(str_ssp).size(16).height(16).vertical_alignment(alignment::Vertical::Center),
                text(&format!("{:04.1}", self.zoom_scale)).size(16).height(16).vertical_alignment(alignment::Vertical::Center),
            ]
            .spacing(10)
            .into()
        }
    }

    impl<'a, Message> From<InfoBar> for Element<'a, Message, Renderer>
    where
        Message: 'a,
    {
        fn from(infobar: InfoBar) -> Self {
            component(infobar)
        }
    }
}
// draw for all states in interaction_stack -
// wire snapping -
// wire persisting -

// port placement
// device serialization, along with wire and port geometry

// device placement and saving, gnd, vdd, ideal res
// netlisting
// running simulations
// wires move, wire/geometry move just one end
// floating nets/ports highlight