//! Schematic GUI page
//! includes paramter editor, toolbar, and the canvas itself

use crate::plot::{ChartElement, Plot};
use crate::transforms::VCTransformFreeAspect;
use crate::{plot, viewport_free_aspect};

use crate::IcedStruct;
use iced::widget::row;
use iced::Element;

#[derive(Debug, Clone)]
pub enum PlotPageMsg {
    ViewportEvt(viewport_free_aspect::CompositeMsg<plot::Msg>),
}

/// schematic
pub struct PlotPage {
    /// viewport
    viewport: viewport_free_aspect::Viewport<Plot<ChartElement>, plot::Msg>,
}
impl Default for PlotPage {
    fn default() -> Self {
        let vct = VCTransformFreeAspect::identity()
            .pre_flip_y()
            .then_scale(10.0, 10.0);
        PlotPage {
            viewport: viewport_free_aspect::Viewport::new(1.0, 1.0, 100.0, vct),
        }
    }
}

impl IcedStruct<PlotPageMsg> for PlotPage {
    fn update(&mut self, msg: PlotPageMsg) {
        match msg {
            PlotPageMsg::ViewportEvt(msgs) => {
                self.viewport.update(msgs);
            }
        }
    }

    fn view(&self) -> Element<PlotPageMsg> {
        let str_ssp = format!(
            "x: {}; y: {}",
            self.viewport.curpos_ssp().x,
            self.viewport.curpos_ssp().y
        );
        let canvas = self.viewport.view().map(PlotPageMsg::ViewportEvt);
        let infobar = row![iced::widget::text(str_ssp)
            .size(16)
            .height(16)
            .vertical_alignment(iced::alignment::Vertical::Center),]
        .spacing(10);

        let schematic = iced::widget::column![canvas, infobar,];

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
