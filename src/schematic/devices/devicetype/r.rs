use crate::transforms::{SSPoint, VSPoint, SSBox};
use super::{Graphics, Port};
use iced::Element;
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "R";

lazy_static! {
    static ref DEFAULT_GRAPHICS: Graphics = Graphics { 
        pts: vec![
            vec![
                VSPoint::new(0., 3.),
                VSPoint::new(0., -3.),
            ],
            vec![
                VSPoint::new(-1., 2.),
                VSPoint::new(-1., -2.),
                VSPoint::new(1., -2.),
                VSPoint::new(1., 2.),
                VSPoint::new(-1., 2.),
            ],
        ],
        ports: vec![
            Port {name: "+", offset: SSPoint::new(0, 3)},
            Port {name: "-", offset: SSPoint::new(0, -3)},
        ], 
        bounds: SSBox::new(SSPoint::new(-2, 3), SSPoint::new(2, -3)), 
    };
}

#[derive(Debug)]
pub struct Raw  {
    raw: String,
}
impl Raw {
    fn new(raw: String) -> Self {
        Raw { raw }
    }
    pub fn set(&mut self, new: String) {
        self.raw = new;
    }
}

#[derive(Debug)]
pub struct SingleValue  {
    value: f32,
}
impl SingleValue {
    fn new(value: f32) -> Self {
        SingleValue { value }
    }
}


#[derive(Debug)]
pub enum ParamR  {
    Raw(Raw),
    Value(SingleValue),
}
impl Default for ParamR {
    fn default() -> Self {
        ParamR::Raw(Raw::new(String::from("1000")))
    }
}
impl ParamR {
    pub fn summary(&self) -> String {
        match self {
            ParamR::Value(v) => {
                std::format!("{}", v.value)
            },
            ParamR::Raw(s) => {
                s.raw.clone()
            },
        }
    }
    pub fn param_editor(&mut self) -> Option<impl ParamEditor + Into<Element<()>>> {
        None::<param_editor::RawParamEditor>
        // match self {
        //     ParamR::Raw(raw) => {
        //         Some(raw.param_editor())
        //     },
        //     ParamR::Value(_) => None,
        // }
    }
}

#[derive(Debug)]
pub struct R {
    pub params: ParamR,
    pub graphics: &'static Graphics,
}
impl R {
    pub fn new() -> R {
        R {params: ParamR::default(), graphics: &DEFAULT_GRAPHICS}
    }
}

pub trait ParamEditor {

}

mod param_editor {
    use iced::widget::{column, text_input, button};
    use iced_lazy::{component, Component};
    use iced::{Length, Element, Renderer};
    use super::ParamEditor;

    #[derive(Debug, Clone)]
    pub enum Evt {
        InputChanged(String),
        InputSubmit,
    }

    pub struct RawParamEditor {
        value: String,
        on_submit: Box<dyn FnMut(String)>,
    }

    impl ParamEditor for RawParamEditor {

    }
    
    impl RawParamEditor {
        pub fn new(
            value: String,
            on_submit: impl FnMut(String) + 'static,
        ) -> Self {
            Self {
                value,
                on_submit: Box::new(on_submit),
            }
        }
    }

    pub fn param_editor(
        value: String,
        on_submit: impl FnMut(String) + 'static,
    ) -> RawParamEditor {
        RawParamEditor::new(value, on_submit)
    }

    impl Component<(), Renderer> for RawParamEditor {
        type State = ();
        type Event = Evt;

        fn update(
            &mut self,
            _state: &mut Self::State,
            event: Evt,
        ) -> Option<()> {
            match event {
                Evt::InputChanged(s) => {
                    self.value = s;
                    None
                },
                Evt::InputSubmit => {
                    Some((self.on_submit)(self.value.clone()))
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

    impl<'a> From<RawParamEditor> for Element<'a, (), Renderer>
    {
        fn from(parameditor: RawParamEditor) -> Self {
            component(parameditor)
        }
    }
}