//! Schematic GUI page
//! includes paramter editor, toolbar, and the canvas itself

use crate::circuit::{Circuit, CircuitElement, Msg};
use crate::schematic;
use crate::viewport::CompositeMsg;

use crate::schematic::{RcRDevice, Schematic};
use crate::{transforms::VCTransform, viewport::Viewport};
use crate::{viewport, IcedStruct};
use iced::widget::canvas::Event;
use iced::widget::{button, row, Row, Text};
use iced::Length;
use std::sync::Arc;

use colored::Colorize;
use iced_aw::{Card, Modal};
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
    ViewportEvt(viewport::CompositeMsg<schematic::Msg<Msg, CircuitElement>>),
    TextInputChanged(String),
    TextInputSubmit,
    CloseModal,
}

/// schematic
pub struct CircuitPage {
    /// viewport
    viewport:
        Viewport<Schematic<Circuit, CircuitElement, Msg>, schematic::Msg<Msg, CircuitElement>>,

    /// tentative net name, used only for display in the infobar
    net_name: Option<String>,
    /// active device - some if only 1 device selected, otherwise is none
    active_element: Option<CircuitElement>,
    /// parameter editor text
    text: String,

    /// spice manager
    spmanager: Arc<SpManager>,
    /// ngspice library
    lib: PkSpice<SpManager>,

    /// show new device
    show_modal: bool,
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
            active_element: Default::default(),
            text: Default::default(),
            spmanager,
            lib,
            show_modal: false,
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
                if let Some(ad) = &self.active_element {
                    match ad {
                        CircuitElement::NetEdge(_) => {}
                        CircuitElement::Device(d) => {
                            d.0.borrow_mut()
                                .class_mut()
                                .set_raw_param(self.text.clone());
                        }
                        CircuitElement::Label(l) => {
                            l.0.borrow_mut().set_name(self.text.clone());
                        }
                    }
                    self.viewport.passive_cache.clear();
                }
            }
            CircuitPageMsg::ViewportEvt(msgs) => {
                match msgs.content_msg {
                    schematic::Msg::Event(
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::D,
                            modifiers: _,
                        }),
                        _,
                    ) => {
                        self.show_modal = !self.show_modal;
                    }
                    schematic::Msg::Event(
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Space,
                            modifiers: _,
                        }),
                        _,
                    ) => {
                        self.viewport.update(CompositeMsg {
                            content_msg: schematic::Msg::ContentMsg(Msg::NetList),
                            viewport_msg: viewport::Msg::None,
                        });
                        self.lib.command("source netlist.cir"); // results pointer array starts at same address
                        self.lib.command("op"); // ngspice recommends sending in control statements separately, not as part of netlist
                        if let Some(pkvecvaluesall) = self.spmanager.tmp.as_ref() {
                            self.viewport.update(CompositeMsg {
                                content_msg: schematic::Msg::ContentMsg(Msg::DcOp(
                                    pkvecvaluesall.clone(),
                                )),
                                viewport_msg: viewport::Msg::None,
                            });
                        }
                    }
                    _ => {
                        self.viewport.update(msgs);
                    }
                }

                // self.active_element = self.viewport.content.active_element().cloned();
                match &self.viewport.content.active_element {
                    Some(ae) => {
                        self.active_element = Some(ae.clone());
                        match ae {
                            CircuitElement::NetEdge(_) => {}
                            CircuitElement::Device(d) => {
                                self.text = d.0.borrow().class().param_summary();
                            }
                            CircuitElement::Label(l) => {
                                self.text = l.0.borrow().read().to_string();
                            }
                        }
                    }
                    None => self.text = String::from(""),
                }

                self.net_name = self.viewport.content.content.infobarstr.take();
            }
            CircuitPageMsg::CloseModal => {
                self.show_modal = false;
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
        let toolbar =
            row![
                button("wire").on_press(CircuitPageMsg::ViewportEvt(viewport::CompositeMsg {
                    content_msg: schematic::Msg::ContentMsg(Msg::Wire),
                    viewport_msg: viewport::Msg::None,
                })),
            ]
            .width(Length::Fill);

        let schematic = iced::widget::column![
            toolbar,
            iced::widget::row![pe, iced::widget::column![canvas, infobar,]]
        ];

        // schematic.into()

        Modal::new(self.show_modal, schematic, || {
            Card::new(
                Text::new("New Device"),
                Text::new(
                    "
                R: Resistor
                V: Voltage Source
                G: Ground
                ",
                ), //Text::new("Zombie ipsum reversus ab viral inferno, nam rick grimes malum cerebro. De carne lumbering animata corpora quaeritis. Summus brains sit​​, morbo vel maleficia? De apocalypsi gorger omero undead survivor dictum mauris. Hi mindless mortuis soulless creaturas, imo evil stalking monstra adventus resi dentevil vultus comedat cerebella viventium. Qui animated corpse, cricket bat max brucks terribilem incessu zomby. The voodoo sacerdos flesh eater, suscitat mortuos comedere carnem virus. Zonbi tattered for solum oculi eorum defunctis go lum cerebro. Nescio brains an Undead zombies. Sicut malus putrid voodoo horror. Nigh tofth eliv ingdead.")
            )
            .foot(Row::new().spacing(10).padding(5).width(Length::Fill))
            .max_width(300.0)
            //.width(Length::Shrink)
            .on_close(CircuitPageMsg::CloseModal)
            .into()
        })
        .backdrop(CircuitPageMsg::CloseModal)
        .on_esc(CircuitPageMsg::CloseModal)
        .into()
    }
}

struct NgModels {
    models: Vec<NgModel>,
}

impl NgModels {
    fn model_definitions(&self) -> String {
        let mut ret = String::new();
        for m in &self.models {
            ret.push_str(&m.model_line())
        }
        ret
    }
}

struct NgModel {
    name: String,
    definition: String,
}

impl NgModel {
    fn model_line(&self) -> String {
        format!(".model {} {}\n", self.name, self.definition)
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
