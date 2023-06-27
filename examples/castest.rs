use iced::{Sandbox, Element, Renderer, widget::{text, canvas, button}, mouse};
use iced_lazy::{Component, component};

#[derive(Debug, Default)]
pub struct DeviceDesigner {

}

#[derive(Debug, Clone, Copy)]
enum DeviceDesignerMsg {
    None,
}

impl DeviceDesigner {
    fn update(&mut self) {
        todo!()
    }

    fn view(&self) -> iced::Element<DeviceDesignerMsg> {
        iced::widget::column![
            canvas(self),
        ].into()
    }
}

impl iced::widget::canvas::Program<DeviceDesignerMsg> for DeviceDesigner {
    type State = DesignerCanvasSt;

    fn draw(
        &self,
        state: &Self::State,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        cursor: iced::widget::canvas::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        todo!()
    }

    fn update(
            &self,
            _state: &mut Self::State,
            _event: iced::widget::canvas::Event,
            _bounds: iced::Rectangle,
            _cursor: iced::widget::canvas::Cursor,
        ) -> (iced::widget::canvas::event::Status, Option<DeviceDesignerMsg>) {
        todo!()
    }

    fn mouse_interaction(
            &self,
            _state: &Self::State,
            _bounds: iced::Rectangle,
            _cursor: iced::widget::canvas::Cursor,
        ) -> mouse::Interaction {
        todo!()
    }
}
/// main program
pub struct Top {
    designer: DeviceDesigner
}

#[derive(Debug)]
pub enum TopMsg {}

impl Sandbox for Top {
    type Message = TopMsg;

    fn new() -> Self {
        todo!()
    }

    fn title(&self) -> String {
        todo!()
    }

    fn update(&mut self, message: Self::Message) {
        todo!()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let d = DesignerComponent{designer: &self.designer};
        d.into()
    }
}

pub struct DesignerComponent<'a> {
    designer: &'a DeviceDesigner
}

pub enum DesignerEvt {
}

impl<'a, Message> Component<Message, Renderer> for DesignerComponent<'a> {
    type State = DeviceDesigner;

    type Event = DesignerEvt;

    fn update(
        &mut self,
        state: &mut Self::State,
        event: Self::Event,
    ) -> Option<Message> {
        todo!()
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Renderer> {
        let c = iced::widget::canvas(self.designer);
        iced::widget::row![
            // c,
            text("text"),
        ]
        .into()
    }
}

impl<'a, Message> From<DesignerComponent<'a>> for Element<'a, Message, Renderer>
where
    Message: 'a,
{
    fn from(designer_component: DesignerComponent<'a>) -> Self {
        component(designer_component)
    }
}

pub enum DesignerCanvasMsg {
}

#[derive(Default)]
pub struct DesignerCanvasSt {
}




fn main() {
}
