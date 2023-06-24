//! Circe
//! Schematic Capture for EDA with ngspice integration

use std::fmt::Debug;
use std::sync::Arc;

mod transforms;
use designer::{Designer, DesignerState};
use transforms::{CSBox, CSPoint, Point, SSPoint, VSPoint};

mod viewport;
use viewport::ViewportState;

mod schematic;
use schematic::{RcRDevice, Schematic, SchematicState};

mod designer;
mod interactable;

use iced::{
    executor, mouse,
    widget::{
        canvas,
        canvas::{
            event::{self, Event},
            Cache, Cursor, Geometry,
        },
        column, row,
    },
    Application, Color, Command, Element, Length, Rectangle, Settings, Size, Theme,
};

use iced_aw::{TabLabel, Tabs};

use colored::Colorize;
use paprika::*;

/// Spice Manager to facillitate interaction with NgSpice
struct SpManager {
    tmp: Option<PkVecvaluesall>,
}

impl SpManager {
    fn new() -> Self {
        SpManager { tmp: None }
    }
}

#[allow(unused_variables)]
impl paprika::PkSpiceManager for SpManager {
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
    fn cb_ctrldexit(&mut self, status: i32, is_immediate: bool, is_quit: bool, id: i32) {}
    fn cb_send_init(&mut self, pkvecinfoall: PkVecinfoall, id: i32) {}
    fn cb_send_data(&mut self, pkvecvaluesall: PkVecvaluesall, count: i32, id: i32) {
        self.tmp = Some(pkvecvaluesall);
    }
    fn cb_bgt_state(&mut self, is_fin: bool, id: i32) {}
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

/// main program
pub struct Circe {
    /// zoom scale of the viewport, used only for display in the infobar
    zoom_scale: f32,
    /// schematic cursor coordinate in schematic space, used only for display in the infobar
    schematic_curpos_ssp: SSPoint,
    /// designer cursor cooridnate in viewport space, used only for display in the infobar
    designer_curpos_vsp: VSPoint,

    /// tentative net name, used only for display in the infobar
    net_name: Option<String>,

    /// parameter editor text
    text: String,

    /// schematic
    schematic: Schematic,
    /// device designer
    designer: Designer,

    /// active device - some if only 1 device selected, otherwise is none
    active_device: Option<RcRDevice>,
    /// spice manager
    spmanager: Arc<SpManager>,
    /// ngspice library
    lib: PkSpice<SpManager>,

    /// active tab index
    active_tab: usize,
}

#[derive(Debug, Clone)]
pub enum Msg {
    NewZoom(f32),
    TextInputChanged(String),
    TextInputSubmit,
    SchematicEvent(Event, SSPoint),
    DesignerEvent(Event, SSPoint),

    TabSel(usize),
}

impl Application for Circe {
    type Executor = executor::Default;
    type Message = Msg;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Msg>) {
        let manager = Arc::new(SpManager::new());
        let mut lib;
        #[cfg(target_family = "windows")]
        {
            lib = PkSpice::<SpManager>::new(std::ffi::OsStr::new("paprika/ngspice.dll")).unwrap();
        }
        #[cfg(target_os = "macos")]
        {
            // retrieve libngspice.dylib from the following possible directories
            let ret = Cmd::new("find")
                .args(&["/usr/lib", "/usr/local/lib"])
                .arg("-name")
                .arg("*libngspice.dylib")
                .stdout(Stdio::piped())
                .output()
                .unwrap_or_else(|_| {
                    eprintln!("Error: Could not find libngspice.dylib. Make sure it is installed.");
                    process::exit(1);
                });
            let path = String::from_utf8(ret.stdout).unwrap();
            lib = PkSpice::<SpManager>::new(&std::ffi::OsString::from(path.trim())).unwrap();
        }
        #[cfg(target_os = "linux")]
        {
            // dynamically retrieves libngspice from system
            let ret = Cmd::new("sh")
                .arg("-c")
                .arg("ldconfig -p | grep ngspice | awk '/.*libngspice.so$/{print $4}'")
                .stdout(Stdio::piped())
                .output()
                .unwrap_or_else(|_| {
                    eprintln!("Error: Could not find libngspice. Make sure it is installed.");
                    process::exit(1);
                });

            let path = String::from_utf8(ret.stdout).unwrap();
            lib = PkSpice::<SpManager>::new(&std::ffi::OsString::from(path.trim())).unwrap();
        }

        lib.init(Some(manager.clone()));
        (
            Circe {
                zoom_scale: 10.0, // would be better to get this from the viewport on startup
                schematic_curpos_ssp: SSPoint::origin(),
                designer_curpos_vsp: VSPoint::origin(),
                net_name: None,

                text: String::from(""),
                schematic: Schematic::default(),
                designer: Designer::default(),
                active_device: None,

                lib,
                spmanager: manager,

                active_tab: 0,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Schematic Prototyping")
    }

    fn update(&mut self, message: Msg) -> Command<Msg> {
        match message {
            Msg::NewZoom(value) => self.zoom_scale = value,
            Msg::TextInputChanged(s) => {
                self.text = s;
            }
            Msg::TextInputSubmit => {
                if let Some(ad) = &self.active_device {
                    ad.0.borrow_mut().class_mut().set(self.text.clone());
                    self.schematic.passive_cache.clear();
                }
            }
            Msg::SchematicEvent(event, ssp) => {
                let (opt_s, clear_passive) = self.schematic.events_handler(event, ssp);
                if clear_passive {
                    self.schematic.passive_cache.clear()
                }
                self.net_name = opt_s;
                self.schematic_curpos_ssp = ssp;
                self.active_device = self.schematic.active_device();
                if let Some(rcrd) = &self.active_device {
                    self.text = rcrd.0.borrow().class().param_summary();
                } else {
                    self.text = String::from("");
                }
                if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key_code: iced::keyboard::KeyCode::Space,
                    modifiers: _,
                }) = event
                {
                    self.lib.command("source netlist.cir"); // results pointer array starts at same address
                    self.lib.command("op"); // ngspice recommends sending in control statements separately, not as part of netlist
                    if let Some(pkvecvaluesall) = self.spmanager.tmp.as_ref() {
                        self.schematic.op(pkvecvaluesall);
                    }
                }
            }
            Msg::DesignerEvent(event, ssp) => {
                let clear_passive = self.designer.events_handler(event, ssp);
                if clear_passive {
                    self.designer.passive_cache.clear()
                }
                self.designer_curpos_vsp = transforms::designer_ssp_to_schematic_vsp(ssp);
                self.text = "".to_string();
            }
            Msg::TabSel(i) => {
                self.active_tab = i;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Msg> {
        let schematic = schematic_component::schematic_component(self);
        let device_designer = device_designer::device_designer(self);

        let tabs = Tabs::with_tabs(
            self.active_tab,
            vec![
                (TabLabel::Text("Schematic".to_string()), schematic),
                (
                    TabLabel::Text("Device Creator".to_string()),
                    device_designer,
                ),
            ],
            Msg::TabSel,
        );

        tabs.into()
    }
}
mod device_designer {
    use iced::{
        alignment,
        widget::{canvas, column, row, text},
        Element, Length,
    };

    pub use crate::Circe;
    use crate::Msg;

    pub fn device_designer(circe: &Circe) -> Element<Msg> {
        let str_vsp = format!(
            "x: {}; y: {}",
            circe.designer_curpos_vsp.x, circe.designer_curpos_vsp.y
        );
        let net_name = circe.net_name.as_deref().unwrap_or_default();

        let canvas = canvas(&circe.designer)
            .width(Length::Fill)
            .height(Length::Fill);
        let dd = row![column![
            canvas,
            row![
                text(str_vsp)
                    .size(16)
                    .height(16)
                    .vertical_alignment(alignment::Vertical::Center),
                text(&format!("{:04.1}", circe.zoom_scale))
                    .size(16)
                    .height(16)
                    .vertical_alignment(alignment::Vertical::Center),
                text(net_name)
                    .size(16)
                    .height(16)
                    .vertical_alignment(alignment::Vertical::Center),
            ]
            .spacing(10)
        ]
        .width(Length::Fill)];
        dd.into()
    }
}

mod schematic_component {
    use iced::{
        alignment,
        widget::{canvas, column, row, text},
        Element, Length,
    };

    pub use crate::Circe;
    use crate::{param_editor::param_editor, Msg};

    pub fn schematic_component(circe: &Circe) -> Element<Msg> {
        let str_ssp = format!(
            "x: {}; y: {}",
            circe.schematic_curpos_ssp.x, circe.schematic_curpos_ssp.y
        );
        let net_name = circe.net_name.as_deref().unwrap_or_default();

        let canvas = canvas(&circe.schematic)
            .width(Length::Fill)
            .height(Length::Fill);
        let pe = param_editor(circe.text.clone(), Msg::TextInputChanged, || {
            Msg::TextInputSubmit
        });
        let schematic = row![
            pe,
            column![
                canvas,
                row![
                    text(str_ssp)
                        .size(16)
                        .height(16)
                        .vertical_alignment(alignment::Vertical::Center),
                    text(&format!("{:04.1}", circe.zoom_scale))
                        .size(16)
                        .height(16)
                        .vertical_alignment(alignment::Vertical::Center),
                    text(net_name)
                        .size(16)
                        .height(16)
                        .vertical_alignment(alignment::Vertical::Center),
                ]
                .spacing(10)
            ]
            .width(Length::Fill)
        ];
        schematic.into()
    }
}

mod param_editor {
    use iced::widget::{button, column, text_input};
    use iced::{Element, Length, Renderer};
    use iced_lazy::{component, Component};

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

        fn update(&mut self, _state: &mut Self::State, event: Evt) -> Option<Message> {
            match event {
                Evt::InputChanged(s) => Some((self.on_change)(s)),
                Evt::InputSubmit => Some((self.on_submit)()),
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
