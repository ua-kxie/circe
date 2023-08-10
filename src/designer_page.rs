//! Designer Schematic GUI page
//! includes paramter editor, toolbar, and the canvas itself

use crate::designer::{Designer, DesignerElement, Msg};
use crate::schematic::{self, Content};
use crate::transforms::VCTransformLockedAspect;

use crate::schematic::Schematic;
use crate::viewport::Viewport;
use crate::{viewport, IcedStruct};
use iced::widget::{button, row};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum DevicePageMsg {
    ViewportEvt(viewport::CompositeMsg<schematic::Msg<Msg, DesignerElement>>),
    TextInputChanged(String),
    TextInputSubmit,
}

/// schematic
pub struct DevicePage {
    /// viewport
    viewport:
        Viewport<Schematic<Designer, DesignerElement, Msg>, schematic::Msg<Msg, DesignerElement>>,

    /// tentative net name, used only for display in the infobar
    net_name: Option<String>,
    /// active device - some if only 1 device selected, otherwise is none
    active_element: Option<DesignerElement>,
    /// parameter editor text
    text: String,
}

impl Default for DevicePage {
    fn default() -> Self {
        let vct = VCTransformLockedAspect::identity()
            .pre_flip_y()
            .then_scale(10.0);
        DevicePage {
            viewport: viewport::Viewport::new(1.0, 100.0, vct),
            net_name: Default::default(),
            active_element: Default::default(),
            text: Default::default(),
        }
    }
}

impl IcedStruct<DevicePageMsg> for DevicePage {
    fn update(&mut self, msg: DevicePageMsg) {
        match msg {
            DevicePageMsg::TextInputChanged(s) => {
                self.text = s;
            }
            DevicePageMsg::TextInputSubmit => {}
            DevicePageMsg::ViewportEvt(msgs) => {
                self.viewport.update(msgs);

                match &self.viewport.content.active_element {
                    Some(_) => {}
                    None => self.text = String::from(""),
                }
            }
        }
    }

    fn view(&self) -> Element<DevicePageMsg> {
        let str_ssp = format!(
            "x: {}; y: {}",
            self.viewport.content.content.curpos_vsp().x,
            self.viewport.content.content.curpos_vsp().y
        );
        let canvas = self.viewport.view().map(DevicePageMsg::ViewportEvt);
        let pe =
            param_editor::param_editor(self.text.clone(), DevicePageMsg::TextInputChanged, || {
                DevicePageMsg::TextInputSubmit
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
                button("wire").on_press(DevicePageMsg::ViewportEvt(viewport::CompositeMsg {
                    content_msg: schematic::Msg::ContentMsg(Msg::Line),
                    viewport_msg: viewport::Msg::None,
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
