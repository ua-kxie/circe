//! Circuit Schematic GUI page
//! includes paramter editor, toolbar, and the canvas itself

use crate::circuit::{Circuit, CircuitElement, Msg};
use crate::schematic;
use crate::transforms::{VCTransformLockedAspect, VSPoint};
use crate::viewport::CompositeMsg;

use crate::schematic::Schematic;
use crate::viewport::Viewport;
use crate::{viewport, IcedStruct};
use iced::keyboard::Modifiers;
use iced::widget::canvas::Event;
use iced::widget::{button, row, Row, Text};
use iced::{Element, Length};
use std::sync::{Arc, Mutex};

use colored::Colorize;
use paprika::*;

/// Spice Manager to facillitate interaction with NgSpice
#[derive(Debug, Default)]
struct SpManager {
    vecvals: Mutex<Vec<PkVecvaluesall>>,
    vecinfo: Option<PkVecinfoall>,
}

impl SpManager {
    fn new() -> Self {
        SpManager::default()
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
    fn cb_send_init(&mut self, pkvecinfoall: PkVecinfoall, id: i32) {
        self.vecinfo = Some(pkvecinfoall);
    }
    fn cb_send_data(&mut self, pkvecvaluesall: PkVecvaluesall, count: i32, id: i32) {
        // this is called every simulation step when running tran
        self.vecvals.try_lock().unwrap().push(pkvecvaluesall);
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
    /// traces from certain simulations e.g. transient
    pub traces: Option<Vec<Vec<VSPoint>>>,

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
        let vct = VCTransformLockedAspect::identity()
            .pre_flip_y()
            .then_scale(10.0);
        CircuitPage {
            viewport: viewport::Viewport::new(1.0, 100.0, vct),
            net_name: Default::default(),
            active_element: Default::default(),
            text: Default::default(),
            spmanager,
            lib,
            traces: None,
            show_modal: false,
        }
    }
}

impl IcedStruct<CircuitPageMsg> for CircuitPage {
    fn update(&mut self, msg: CircuitPageMsg) {
        const NO_MODIFIER: Modifiers = Modifiers::empty();
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
                            modifiers: NO_MODIFIER,
                        }),
                        _,
                    ) => {
                        self.show_modal = !self.show_modal;
                    }
                    schematic::Msg::Event(
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Space,
                            modifiers: NO_MODIFIER,
                        }),
                        _,
                    ) => {
                        self.viewport.update(CompositeMsg {
                            content_msg: schematic::Msg::ContentMsg(Msg::NetList),
                            viewport_msg: viewport::Msg::None,
                        });
                        self.lib.command("source netlist.cir"); // results pointer array starts at same address
                        self.lib.command("op"); // ngspice recommends sending in control statements separately, not as part of netlist
                        if let Some(pkvecvaluesall) =
                            self.spmanager.vecvals.try_lock().unwrap().pop()
                        {
                            self.viewport.update(CompositeMsg {
                                content_msg: schematic::Msg::ContentMsg(Msg::DcOp(
                                    pkvecvaluesall.clone(),
                                )),
                                viewport_msg: viewport::Msg::None,
                            });
                        }
                    }
                    schematic::Msg::Event(
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Space,
                            modifiers: Modifiers::CTRL,
                        }),
                        _,
                    ) => {
                        self.viewport.update(CompositeMsg {
                            content_msg: schematic::Msg::ContentMsg(Msg::NetList),
                            viewport_msg: viewport::Msg::None,
                        });
                        self.lib.command("source netlist.cir"); // results pointer array starts at same address
                        self.lib.command("ac lin 0 60 60"); // ngspice recommends sending in control statements separately, not as part of netlist
                        if let Some(pkvecvaluesall) =
                            self.spmanager.vecvals.try_lock().unwrap().pop()
                        {
                            self.viewport.update(CompositeMsg {
                                content_msg: schematic::Msg::ContentMsg(Msg::Ac(
                                    pkvecvaluesall.clone(),
                                )),
                                viewport_msg: viewport::Msg::None,
                            });
                        }
                    }
                    schematic::Msg::Event(
                        Event::Keyboard(iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::T,
                            modifiers: iced::keyboard::Modifiers::SHIFT,
                        }),
                        _,
                    ) => {
                        self.viewport.update(CompositeMsg {
                            content_msg: schematic::Msg::ContentMsg(Msg::NetList),
                            viewport_msg: viewport::Msg::None,
                        });
                        self.lib.command("source netlist.cir"); // results pointer array starts at same address
                        self.spmanager.vecvals.try_lock().unwrap().clear();
                        self.lib.command("tran 10u 1m"); // ngspice recommends sending in control statements separately, not as part of netlist

                        let pk_results = self.spmanager.vecvals.try_lock().unwrap();

                        let trace_count = pk_results.first().unwrap().count as usize;
                        let mut results: Vec<Vec<VSPoint>> = Vec::with_capacity(trace_count);
                        for _ in 0..trace_count {
                            results.push(Vec::with_capacity(pk_results.len()));
                        }

                        let x_i = pk_results
                            .first()
                            .unwrap()
                            .vecsa
                            .iter()
                            .position(|x| x.name == "time")
                            .unwrap();
                        for step_val in pk_results.iter() {
                            for (trace_i, trace_val) in step_val.vecsa.iter().enumerate() {
                                results[trace_i].push(VSPoint::new(
                                    step_val.vecsa[x_i].creal as f32,
                                    trace_val.creal as f32,
                                ));
                            }
                        }
                        results.remove(x_i);

                        self.traces = Some(results);
                    }
                    _ => {
                        self.viewport.update(msgs);
                    }
                }

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

    fn view(&self) -> Element<CircuitPageMsg> {
        let str_ssp = format!(
            "x: {}; y: {}",
            self.viewport.content.content.curpos_ssp().x,
            self.viewport.content.content.curpos_ssp().y
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

        iced_aw::Modal::new(
            self.show_modal,
            schematic,
            iced_aw::Card::new(
                Text::new("New Device"),
                Text::new(
                    "
                R: Resistor
                V: Voltage Source
                G: Ground
                P: PMOS
                N: NMOS
                ",
                ),
            )
            .foot(Row::new().spacing(10).padding(5).width(Length::Fill))
            .max_width(300.0)
            //.width(Length::Shrink)
            .on_close(CircuitPageMsg::CloseModal),
        )
        .backdrop(CircuitPageMsg::CloseModal)
        .on_esc(CircuitPageMsg::CloseModal)
        .into()
    }
}

mod param_editor {
    use iced::widget::{button, column, component, text_input, Component};
    use iced::{Element, Length, Renderer};

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
