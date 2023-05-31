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
use iced::widget::{canvas, column, row, button};
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
             size: (600, 500), 
             ..iced::window::Settings::default()
            },
        antialiasing: true,
        ..Settings::default()
    })
}

struct Circe {
    zoom_scale: f32,
    curpos_ssp: SSPoint,
    net_name: String,

    active_cache: Cache,
    passive_cache: Cache,
    background_cache: Cache,
}

#[derive(Debug, Clone)]
pub enum Msg {
    NewCurpos(SSPoint),
    NewZoom(f32),
    NetName(Option<String>),
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
                curpos_ssp: SSPoint::origin(),
                net_name: String::from(""),

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
            Msg::NetName(opts) => {
                if let Some(s) = opts {
                    self.net_name = s;
                } else {
                    self.net_name.clear();
                }
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Msg> {
        let canvas = canvas(self as &Self)
            .width(Length::Fill)
            .height(Length::Fill);
        let infobar = infobar(self.curpos_ssp, self.zoom_scale, self.net_name.clone());
        row![
            button("placeholder"),
            column![
                canvas,
                infobar,
            ]
            .width(Length::Fill),
        ]
        .into()
    }
}

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
        let vstate = sttup.0.state.clone();
        let mut msg = None;

        if let Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code, modifiers}) = event {
            // keys
            match (vstate, key_code, modifiers.bits(), curpos) {
                (_, iced::keyboard::KeyCode::T, 0, _) => {
                    sttup.1.key_test();
                    self.passive_cache.clear();
                },
                (_, iced::keyboard::KeyCode::F, 0, _) => {
                    let vsb = sttup.1.bounding_box().inflate(5., 5.);
                    sttup.0.display_bounds(
                        CSBox::from_points([CSPoint::origin(), CSPoint::new(bounds.width, bounds.height)]), 
                        vsb,
                    );
                    self.passive_cache.clear();
                },
                _ => {},
            }
        }
        
        if let Some(curpos_csp) = curpos.map(|x| Point::from(x).into()) {
            let (msg0, clear_passive0) = sttup.0.events_handler(event, curpos_csp, bounds);
            let (msg1, clear_passive1) = sttup.1.events_handler(event, sttup.0.curpos_vsp(), sttup.0.curpos_ssp());
            msg = msg0.or(msg1);
            if clear_passive0 || clear_passive1 { self.passive_cache.clear() }
        }
        self.active_cache.clear();
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
                (ViewportState::Panning(_), _) => mouse::Interaction::Grabbing,
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
        curpos_ssp: SSPoint,
        zoom_scale: f32,
        net_name: String,
    }
    
    impl InfoBar {
        pub fn new(
            curpos_ssp: SSPoint,
            zoom_scale: f32,
            net_name: String,
        ) -> Self {
            Self {
                curpos_ssp,
                zoom_scale,
                net_name,
            }
        }
    }

    pub fn infobar(
        curpos_ssp: SSPoint,
        zoom_scale: f32,
        net_name: String,
    ) -> InfoBar {
        InfoBar::new(curpos_ssp, zoom_scale, net_name)
    }

    impl<Message> Component<Message, Renderer> for InfoBar {
        type State = ();
        type Event = ();

        fn update(
            &mut self,
            _state: &mut Self::State,
            _event: (),
        ) -> Option<Message> {
            None
        }
        fn view(&self, _state: &Self::State) -> Element<(), Renderer> {
            let str_ssp;
            str_ssp = format!("x: {}; y: {}", self.curpos_ssp.x, self.curpos_ssp.y);

            row![
                text(str_ssp).size(16).height(16).vertical_alignment(alignment::Vertical::Center),
                text(&format!("{:04.1}", self.zoom_scale)).size(16).height(16).vertical_alignment(alignment::Vertical::Center),
                text(&self.net_name).size(16).height(16).vertical_alignment(alignment::Vertical::Center),
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
