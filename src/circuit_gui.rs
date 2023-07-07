//! Schematic GUI page
//! includes paramter editor, toolbar, and the canvas itself

use crate::circuit::{Circuit, CircuitElement, CircuitMsg};
use crate::circuit_gui;

use crate::schematic::{RcRDevice, Schematic, SchematicMsg};
use crate::viewport::ContentMsgs;
use crate::{transforms::VCTransform, viewport::Viewport};
use crate::{viewport, IcedStruct};
use iced::widget::{button, row};
use iced::Length;
use std::sync::Arc;

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

#[derive(Debug, Clone)]
pub enum CircuitPageMsg {
    ViewportEvt(viewport::ContentMsgs<CircuitMsg>),
    TextInputChanged(String),
    TextInputSubmit,
}

/// schematic
pub struct CircuitPage {
    /// viewport
    viewport: Viewport<Schematic<Circuit, CircuitElement>, CircuitMsg>,

    circuit: Circuit,

    /// tentative net name, used only for display in the infobar
    net_name: Option<String>,
    /// active device - some if only 1 device selected, otherwise is none
    active_device: Option<RcRDevice>,
    /// parameter editor text
    text: String,

    /// spice manager
    spmanager: Arc<SpManager>,
    /// ngspice library
    lib: PkSpice<SpManager>,
}
impl Default for CircuitPage {
    fn default() -> Self {
        let spmanager = Arc::new(SpManager::new());
        let mut lib;
        #[cfg(target_family = "windows")]
        {
            lib = PkSpice::<SpManager>::new(std::ffi::OsStr::new("paprika/ngspice.dll")).unwrap();
        }
        #[cfg(target_os = "macos")]
        {
            // retrieve libngspice.dylib from the following possible directories
            let ret = std::process::Command::new("find")
                .args(&["/usr/lib", "/usr/local/lib"])
                .arg("-name")
                .arg("*libngspice.dylib")
                .stdout(std::process::Stdio::piped())
                .output()
                .unwrap_or_else(|_| {
                    eprintln!("Error: Could not find libngspice.dylib. Make sure it is installed.");
                    std::process::exit(1);
                });
            let path = String::from_utf8(ret.stdout).unwrap();
            lib = PkSpice::<SpManager>::new(&std::ffi::OsString::from(path.trim())).unwrap();
        }
        #[cfg(target_os = "linux")]
        {
            // dynamically retrieves libngspice from system
            let ret = std::process::Command::new("sh")
                .arg("-c")
                .arg("ldconfig -p | grep ngspice | awk '/.*libngspice.so$/{print $4}'")
                .stdout(std::process::Stdio::piped())
                .output()
                .unwrap_or_else(|_| {
                    eprintln!("Error: Could not find libngspice. Make sure it is installed.");
                    std::process::exit(1);
                });

            let path = String::from_utf8(ret.stdout).unwrap();
            lib = PkSpice::<SpManager>::new(&std::ffi::OsString::from(path.trim())).unwrap();
        }
        lib.init(Some(spmanager.clone()));
        let vct = VCTransform::identity().then_scale(10.0, -10.0);
        CircuitPage {
            viewport: viewport::Viewport::new(1.0, 1.0, 100.0, vct),
            net_name: Default::default(),
            active_device: Default::default(),
            text: Default::default(),
            spmanager,
            lib,
            circuit: Default::default(),
        }
    }
}

impl IcedStruct<CircuitPageMsg> for CircuitPage {
    fn update(&mut self, msg: CircuitPageMsg) {
        match msg {
            CircuitPageMsg::TextInputChanged(s) => {
                self.text = s;
            }
            CircuitPageMsg::TextInputSubmit => {
                if let Some(ad) = &self.active_device {
                    ad.0.borrow_mut().class_mut().set(self.text.clone());
                    self.viewport.passive_cache.clear();
                }
            }
            CircuitPageMsg::ViewportEvt(msgs) => {
                self.viewport.update(msgs);

                if let Some(CircuitMsg::DcOp) = msgs.content_msg {
                    self.circuit.netlist();
                    self.lib.command("source netlist.cir"); // results pointer array starts at same address
                    self.lib.command("op"); // ngspice recommends sending in control statements separately, not as part of netlist
                    if let Some(pkvecvaluesall) = self.spmanager.tmp.as_ref() {
                        self.circuit.op(pkvecvaluesall);
                    }
                }

                self.active_device = self.circuit.active_device();
                if let Some(rcrd) = &self.active_device {
                    self.text = rcrd.0.borrow().class().param_summary();
                } else {
                    self.text = String::from("");
                }

                self.net_name = self.circuit.infobarstr.take();
            }
        }
    }

    fn view(&self) -> iced::Element<CircuitPageMsg> {
        let str_ssp = format!(
            "x: {}; y: {}",
            self.viewport.curpos_ssp().x,
            self.viewport.curpos_ssp().y
        );
        let canvas = self.viewport.view().map(CircuitPageMsg::ViewportEvt);
        let pe =
            param_editor::param_editor(self.text.clone(), CircuitPageMsg::TextInputChanged, || {
                CircuitPageMsg::TextInputSubmit
            });
        // let mut pe = text("");
        // if let Some(active_device) = &self.active_device {
        //     active_device.0.borrow().class().
        // }
        let infobar = row![
            iced::widget::text(str_ssp)
                .size(16)
                .height(16)
                .vertical_alignment(iced::alignment::Vertical::Center),
            iced::widget::text(&format!("{:04.1}", self.viewport.vc_scale()))
                .size(16)
                .height(16)
                .vertical_alignment(iced::alignment::Vertical::Center),
            iced::widget::text(self.net_name.as_deref().unwrap_or_default())
                .size(16)
                .height(16)
                .vertical_alignment(iced::alignment::Vertical::Center),
        ]
        .spacing(10);
        let toolbar = row![
            button("wire").on_press(CircuitPageMsg::ViewportEvt(ContentMsgs {
                content_msg: Some(CircuitMsg::Wire),
                viewport_msg: None
            })),
        ]
        .width(Length::Fill);

        let schematic = iced::widget::column![
            toolbar,
            iced::widget::row![pe, iced::widget::column![canvas, infobar,]]
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
                button("enter").on_press(Evt::InputSubmit),
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
