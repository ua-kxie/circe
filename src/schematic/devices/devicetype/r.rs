//! device definition for resistors (RXXXX)

use super::super::params;
use super::{Graphics, Port};
use crate::transforms::{SSBox, SSPoint, VSPoint};
use iced::Element;
use lazy_static::lazy_static;

pub const ID_PREFIX: &str = "R";

lazy_static! {
    static ref DEFAULT_GRAPHICS: Graphics = Graphics {
        pts: vec![
            vec![VSPoint::new(0., 3.), VSPoint::new(0., -3.),],
            vec![
                VSPoint::new(-1., 2.),
                VSPoint::new(-1., -2.),
                VSPoint::new(1., -2.),
                VSPoint::new(1., 2.),
                VSPoint::new(-1., 2.),
            ],
        ],
        circles: vec![],
        ports: vec![
            Port {
                name: "+".to_string(),
                offset: SSPoint::new(0, 3)
            },
            Port {
                name: "-".to_string(),
                offset: SSPoint::new(0, -3)
            },
        ],
        bounds: SSBox::new(SSPoint::new(-2, 3), SSPoint::new(2, -3)),
    };
}

/// Enumerates the different ways to specifify parameters for a resistor
#[derive(Debug, Clone)]
pub enum ParamR {
    /// specify the spice line directly (after id and port connections)
    Raw(params::Raw),
    /// specify the spice line by a single value
    Value(params::SingleValue),
}
impl Default for ParamR {
    fn default() -> Self {
        ParamR::Raw(params::Raw::new(String::from("1000")))
    }
}
impl ParamR {
    pub fn summary(&self) -> String {
        match self {
            ParamR::Value(v) => {
                std::format!("{}", v.value)
            }
            ParamR::Raw(s) => s.raw.clone(),
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

/// resistor device class
#[derive(Debug, Clone)]
pub struct R {
    /// parameters of the resistor
    pub params: ParamR,
    /// graphic representation of the resistor
    pub graphics: &'static Graphics,
}
impl R {
    pub fn new() -> R {
        R {
            params: ParamR::default(),
            graphics: &DEFAULT_GRAPHICS,
        }
    }
}

pub trait ParamEditor {}

mod param_editor {
    use super::ParamEditor;
    use iced::widget::{button, column, text_input};
    use iced::{Element, Length, Renderer};
    use iced_lazy::{component, Component};

    #[derive(Debug, Clone)]
    pub enum Evt {
        InputChanged(String),
        InputSubmit,
    }

    pub struct RawParamEditor {
        value: String,
        on_submit: Box<dyn FnMut(String)>,
    }

    impl ParamEditor for RawParamEditor {}

    impl RawParamEditor {
        pub fn new(value: String, on_submit: impl FnMut(String) + 'static) -> Self {
            Self {
                value,
                on_submit: Box::new(on_submit),
            }
        }
    }

    pub fn param_editor(value: String, on_submit: impl FnMut(String) + 'static) -> RawParamEditor {
        RawParamEditor::new(value, on_submit)
    }

    impl Component<(), Renderer> for RawParamEditor {
        type State = ();
        type Event = Evt;

        fn update(&mut self, _state: &mut Self::State, event: Evt) -> Option<()> {
            match event {
                Evt::InputChanged(s) => {
                    self.value = s;
                    None
                }
                Evt::InputSubmit => Some((self.on_submit)(self.value.clone())),
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

    impl<'a> From<RawParamEditor> for Element<'a, (), Renderer> {
        fn from(parameditor: RawParamEditor) -> Self {
            component(parameditor)
        }
    }
}
