mod nets;
mod devices;

mod transforms;
use std::fmt::Debug;

use transforms::{Point, CSPoint, VSPoint, CSBox, VSBox, SSPoint};
mod viewport;
use viewport::ViewportState;
mod schematic;
use schematic::{Schematic, SchematicState};

use iced::keyboard::Modifiers;
use iced::{executor, Size, alignment};
use iced::widget::canvas::{
    stroke, Cache, Cursor, Geometry, LineCap, Path, Stroke, LineDash, Frame,
};
use iced::widget::{canvas, container, text, column};
use iced::{
    Application, Color, Command, Element, Length, Rectangle, Settings,
    Subscription, Theme, Vector,
};
use iced::widget::canvas::event::{self, Event};
use iced::mouse;
use euclid::{Vector2D, Transform2D, Box2D, Point2D};

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
    schematic: Schematic,
    infotext: String,

    active_cache: Cache,
    passive_cache: Cache,
    background_cache: Cache,
}

#[derive(Debug, Clone)]
enum Msg {
    NewCurpos(Option<(VSPoint, SSPoint)>),
    LeftClick(SSPoint),
    Wire,
    Cycle,
    Test,
    Esc,
    Del,
}

impl Application for Circe {
    type Executor = executor::Default;
    type Message = Msg;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Msg>) {
        (
            Circe {
                schematic: Default::default(),
                infotext: String::from(""),

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
            Msg::NewCurpos(opt_curpos) => {
                self.schematic.curpos_update(opt_curpos);
                self.infotext.clear();
                if let Some((_vsp, curpos_ssp)) = opt_curpos {
                    self.infotext.push_str(&format!("{:?}", curpos_ssp)); 
                }
            }
            Msg::LeftClick(ssp) => {
                self.schematic.left_click(ssp);
            },
            Msg::Wire => {
                self.schematic.key_wire();
            },
            Msg::Cycle => {
                self.schematic.key_cycle();
            },
            Msg::Test => {
                self.schematic.key_test();
            },
            Msg::Esc => {
                self.schematic.key_esc();
            },
            Msg::Del => {
                self.schematic.key_del();
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Msg> {
        let canvas = canvas(self as &Self)
            .width(Length::Fill)
            .height(Length::Fill);
        let infobar = text(&self.infotext).size(16).height(16).vertical_alignment(alignment::Vertical::Center);

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
    type State = Viewport;

    fn update(
        &self,
        viewport: &mut Viewport,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Msg>) {
        let curpos = cursor.position_in(&bounds);
        let state = &viewport.state;
        let mut msg = None;
        match (state, event, curpos) {
            // clicking
            (_, Event::Mouse(ButtonPressed(Left)), Some(_)) => {
                if let Some(ssp) = viewport.curpos_ssp() {
                    self.passive_cache.clear();
                    msg = Some(Msg::LeftClick(ssp));
                }
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
            }

            // keys
            (vstate, Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code, modifiers}), curpos) => { 
                match (vstate, key_code, modifiers.bits(), curpos) {
                    (_, iced::keyboard::KeyCode::W, 0, _) => {
                        msg = Some(Msg::Wire);
                    },
                    (_, iced::keyboard::KeyCode::C, 0, _) => {
                        msg = Some(Msg::Cycle);
                        self.active_cache.clear();
                        self.passive_cache.clear();
                    },
                    (_, iced::keyboard::KeyCode::T, 0, _) => {
                        msg = Some(Msg::Test);
                        self.active_cache.clear();
                        self.passive_cache.clear();
                    },
                    (_, iced::keyboard::KeyCode::F, 0, _) => {
                        let vsb = self.schematic.bounding_box().inflate(5., 5.);
                        viewport.display_bounds(
                            CSBox::from_points([CSPoint::origin(), CSPoint::new(bounds.width, bounds.height)]), 
                            vsb,
                        );
                        self.active_cache.clear();
                        self.passive_cache.clear();
                    },
                    (_, iced::keyboard::KeyCode::Escape, 0, _) => {
                        msg = Some(Msg::Esc);
                        self.active_cache.clear();
                        self.passive_cache.clear();
                    },
                    (_, iced::keyboard::KeyCode::Delete, 0, _) => {
                        msg = Some(Msg::Del);
                        self.active_cache.clear();
                        self.passive_cache.clear();
                    },
                    _ => {},
                }
            }

            (vstate, Event::Mouse(mouse::Event::CursorMoved { position }), opt_csp) => {
                let opt_csp = opt_csp.map(|p| Point::from(p).into());
                self.active_cache.clear();
                if let ViewportState::Panning = vstate {
                    self.passive_cache.clear();
                }
                viewport.curpos_update(opt_csp);
                msg = Some(Msg::NewCurpos(viewport.curpos_vs_ss()))
            }
            _ => {}
        }
        (event::Status::Ignored, msg)
    }

    fn draw(
        &self,
        viewport: &Viewport,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        let active = self.active_cache.draw(bounds.size(), |frame| {
            self.schematic.draw_active(viewport.vc_transform(), viewport.vc_scale(), frame);
            viewport.draw_cursor(frame);

            if let ViewportState::NewView(vsp0, vsp1) = viewport.state {
                let csp0 = viewport.vc_transform().transform_point(vsp0);
                let csp1 = viewport.vc_transform().transform_point(vsp1);
                let selsize = Size{width: csp1.x - csp0.x, height: csp1.y - csp0.y};
                let f = canvas::Fill {
                    style: canvas::Style::Solid(if selsize.height > 0. {Color::from_rgba(1., 0., 0., 0.1)} else {Color::from_rgba(0., 0., 1., 0.1)}),
                    ..canvas::Fill::default()
                };
                frame.fill_rectangle(Point::from(csp0).into(), selsize, f);
            }
        });

        let passive = self.passive_cache.draw(bounds.size(), |frame| {
            viewport.draw_grid(frame, Box2D::new(CSPoint::origin(), Point2D::from([bounds.width, bounds.height])));
            self.schematic.draw_passive(viewport.vc_transform(), viewport.vc_scale(), frame);
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
        viewport: &Viewport,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(&bounds) {
            match (&viewport.state, &self.schematic.state) {
                (ViewportState::Panning, _) => mouse::Interaction::Grabbing,
                (ViewportState::None, SchematicState::Idle(_)) => mouse::Interaction::default(),
                (ViewportState::None, SchematicState::Wiring(_)) => mouse::Interaction::Crosshair,
                _ => mouse::Interaction::default(),
            }
        } else {
            mouse::Interaction::default()
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