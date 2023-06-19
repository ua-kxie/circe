mod transforms;
use std::fmt::Debug;
use std::sync::Arc;

use transforms::{Point, CSPoint, CSBox, SSPoint};
mod viewport;
use viewport::ViewportState;
mod schematic;
use schematic::{Schematic, SchematicState, RcRDevice};

use iced::{executor, Size};
use iced::widget::canvas::{
    Cache, Cursor, Geometry,
};
use iced::widget::{canvas, column, row, button, text};
use iced::{
    Application, Color, Command, Element, Length, Rectangle, Settings,
    Theme,
};
use iced::widget::canvas::event::{self, Event};
use iced::mouse;
use euclid::{Box2D, Point2D};
use infobar::infobar;
use param_editor::{param_editor, ParamEditor};

use paprika::*;
use colored::Colorize;
struct SpManager{
    tmp: Option<PkVecvaluesall>,
}

impl SpManager {
    fn new() -> Self {
        SpManager { tmp: None }
    }
}

#[allow(unused_variables)]
impl paprika::PkSpiceManager for SpManager{
    fn cb_send_char(&mut self, msg: String, id: i32) {
        let opt = msg.split_once(' ');
        let (token, msgs) = match opt {
            Some(tup) => (tup.0, tup.1),
            None => (msg.as_str(), msg.as_str()),
        };
        let msgc = match token {
            "stdout" => msgs.green(),
            "stderr" => msgs.red(),
            _ => msg.magenta().strikethrough(),
        };
        println!("{}", msgc);
    }
    fn cb_send_stat(&mut self, msg: String, id: i32) {
        println!("{}", msg.blue());
    }
    fn cb_ctrldexit(&mut self, status: i32, is_immediate: bool, is_quit: bool, id: i32) {
    }
    fn cb_send_init(&mut self, pkvecinfoall: PkVecinfoall, id: i32) {
    }
    fn cb_send_data(&mut self, pkvecvaluesall: PkVecvaluesall, count: i32, id: i32) {
        self.tmp = Some(pkvecvaluesall);
    }
    fn cb_bgt_state(&mut self, is_fin: bool, id: i32) {
    }
}

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
    net_name: Option<String>,

    active_cache: Cache,
    passive_cache: Cache,
    background_cache: Cache,

    text: String,
    schematic: Schematic,
    active_device: Option<RcRDevice>,

    spmanager: Arc<SpManager>,
    lib: PkSpice<SpManager>,
    libpath: String,
}

#[derive(Debug, Clone)]
pub enum Msg {
    NewZoom(f32),

    TextInputChanged(String),
    TextInputSubmit,
    CanvasEvent(Event, SSPoint),

    LoadPressed,
}

impl Application for Circe {
    type Executor = executor::Default;
    type Message = Msg;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Msg>) {
        let manager = Arc::new(SpManager::new());
        let mut lib = PkSpice::<SpManager>::new(std::ffi::OsStr::new("paprika/ngspice.dll")).unwrap();
        lib.init(Some(manager.clone()));
        (
            Circe {
                zoom_scale: 10.0,  // would be better to get this from the viewport on startup
                curpos_ssp: SSPoint::origin(),
                net_name: None,

                active_cache: Default::default(),
                passive_cache: Default::default(),
                background_cache: Default::default(),

                text: String::from(""),
                schematic: Schematic::default(),
                active_device: None,

                lib,
                spmanager: manager,
                libpath: String::from("paprika/ngspice.dll"),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Schematic Prototyping")
    }

    fn update(&mut self, message: Msg) -> Command<Msg> {
        match message {
            Msg::NewZoom(value) => {
                self.zoom_scale = value
            },
            Msg::TextInputChanged(s) => {
                self.text = s;
            },
            Msg::TextInputSubmit => {
                if let Some(ad) = &self.active_device {
                    ad.0.borrow_mut().class_mut().set(self.text.clone());
                    self.passive_cache.clear();
                }
            },
            Msg::CanvasEvent(event, ssp) => {
                let (opt_s, clear_passive) = self.schematic.events_handler(event, ssp);
                if clear_passive {self.passive_cache.clear()}
                self.net_name = opt_s;
                self.curpos_ssp = ssp;
                self.active_device = self.schematic.active_device();
                if let Some(rcrd) = &self.active_device {
                    self.text = rcrd.0.borrow().class().param_summary();
                } else {
                    self.text = String::from("");
                }
                if let Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code: iced::keyboard::KeyCode::Space, modifiers: _}) = event {
                    self.lib.command("source netlist.cir");  // results pointer array starts at same address
                    self.lib.command("op");  // ngspice recommends sending in control statements separately, not as part of netlist
                    self.schematic.op(self.spmanager.tmp.as_ref().unwrap());
                }
            },
            Msg::LoadPressed => {
                self.lib.command("source netlist.cir");  // results pointer array starts at same address
                self.lib.command("op");  // ngspice recommends sending in control statements separately, not as part of netlist
                self.schematic.op(self.spmanager.tmp.as_ref().unwrap());
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Msg> {
        let canvas = canvas(self as &Self)
            .width(Length::Fill)
            .height(Length::Fill);
        let infobar = infobar(self.curpos_ssp, self.zoom_scale, self.net_name.clone());
        let pe = param_editor(self.text.clone(), Msg::TextInputChanged, || {Msg::TextInputSubmit});
        column![
            row![
                text(&self.libpath).size(24).width(Length::Fill),
                button("Load").on_press(Msg::LoadPressed),
            ],
            row![
                pe,
                column![
                    canvas,
                    infobar,
                ]
                .width(Length::Fill),
            ]
        ]
        .into()
    }
}

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
        let vstate = viewport.state.clone();
        let mut msg = None;
        
        if let Some(curpos_csp) = curpos.map(|x| Point::from(x).into()) {
            if let Event::Keyboard(iced::keyboard::Event::KeyPressed{key_code, modifiers}) = event {
                if let (_, iced::keyboard::KeyCode::F, 0, _) = (vstate, key_code, modifiers.bits(), curpos) {
                    let vsb = self.schematic.bounding_box().inflate(5., 5.);
                    viewport.display_bounds(
                        CSBox::from_points([CSPoint::origin(), CSPoint::new(bounds.width, bounds.height)]), 
                        vsb,
                    );
                    self.passive_cache.clear();
                }
            }

            let (msg0, clear_passive0, processed) = viewport.events_handler(event, curpos_csp, bounds);
            if !processed {
                msg = Some(Msg::CanvasEvent(event, viewport.curpos_ssp()));
            } else {
                if clear_passive0 { self.passive_cache.clear() }
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
        viewport: &Viewport,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
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
                (ViewportState::Panning(_), _) => mouse::Interaction::Grabbing,
                (ViewportState::None, SchematicState::Idle) => mouse::Interaction::default(),
                (ViewportState::None, SchematicState::Wiring(_)) => mouse::Interaction::Crosshair,
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
        net_name: Option<String>,
    }
    
    impl InfoBar {
        pub fn new(
            curpos_ssp: SSPoint,
            zoom_scale: f32,
            net_name: Option<String>,
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
        net_name: Option<String>,
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
            let str_ssp = format!("x: {}; y: {}", self.curpos_ssp.x, self.curpos_ssp.y);
            let s = self.net_name.as_deref().unwrap_or_default();
            row![
                text(str_ssp).size(16).height(16).vertical_alignment(alignment::Vertical::Center),
                text(&format!("{:04.1}", self.zoom_scale)).size(16).height(16).vertical_alignment(alignment::Vertical::Center),
                text(s).size(16).height(16).vertical_alignment(alignment::Vertical::Center),
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

mod param_editor {
    use iced::widget::{column, text_input, button};
    use iced_lazy::{component, Component};
    use iced::{Length, Element, Renderer};

    #[derive(Debug, Clone)]
    pub enum Evt {
        InputChanged(String),
        InputSubmit,
    }

    pub struct ParamEditor<Message> {
        value: String,
        on_change: Box<dyn Fn(String) -> Message>,
        on_submit: Box<dyn Fn() -> Message>,
    }
    
    impl<Message> ParamEditor<Message> {
        pub fn new(
            value: String,
            on_change: impl Fn(String) -> Message + 'static,
            on_submit: impl Fn() -> Message + 'static,
        ) -> Self {
            Self {
                value,
                on_change: Box::new(on_change),
                on_submit: Box::new(on_submit),
            }
        }
    }

    pub fn param_editor<Message>(
        value: String,
        on_change: impl Fn(String) -> Message + 'static,
        on_submit: impl Fn() -> Message + 'static,
    ) -> ParamEditor<Message> {
        ParamEditor::new(value, on_change, on_submit)
    }

    impl<Message> Component<Message, Renderer> for ParamEditor<Message> {
        type State = ();
        type Event = Evt;

        fn update(
            &mut self,
            _state: &mut Self::State,
            event: Evt,
        ) -> Option<Message> {
            match event {
                Evt::InputChanged(s) => {
                    Some((self.on_change)(s))
                },
                Evt::InputSubmit => {
                    Some((self.on_submit)())
                },
            }
        }
        fn view(&self, _state: &Self::State) -> Element<Evt, Renderer> {
            column![
                text_input("", &self.value)
                .width(50)
                .on_input(Evt::InputChanged)
                .on_submit(Evt::InputSubmit),
                button("enter"),
            ]
            .width(Length::Shrink)
            .into()
        }
    }

    impl<'a, Message> From<ParamEditor<Message>> for Element<'a, Message, Renderer>
    where
        Message: 'a,
    {
        fn from(parameditor: ParamEditor<Message>) -> Self {
            component(parameditor)
        }
    }
}
