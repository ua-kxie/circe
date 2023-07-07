//! Circe
//! Schematic Capture for EDA with ngspice integration

use std::fmt::Debug;

mod transforms;
// use designer::DeviceDesigner;
mod viewport;

mod circuit_gui;
mod circuit;
mod schematic;
use circuit_gui::Circuit;

// mod designer;
mod interactable;

use iced::{executor, Application, Command, Element, Settings, Theme};

use iced_aw::{TabLabel, Tabs};

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
    /// schematic
    schematic: Circuit,
    /// intended for dev use for now, can be recycled for user use to design subcircuit (.model) devices
    // designer: DeviceDesigner,

    /// active tab index
    active_tab: usize,
}

#[derive(Debug, Clone)]
pub enum Msg {
    // DeviceDesignerMsg(designer::DeviceDesignerMsg),
    SchematicMsg(circuit_gui::CircuitPageMsg),

    TabSel(usize),
}

impl Application for Circe {
    type Executor = executor::Default;
    type Message = Msg;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Msg>) {
        (
            Circe {
                schematic: Circuit::default(),
                // designer: DeviceDesigner::default(),
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
            Msg::TabSel(i) => {
                self.active_tab = i;
            }
            // Msg::DeviceDesignerMsg(device_designer_msg) => {
            //     self.designer.update(device_designer_msg);
            // }
            Msg::SchematicMsg(schematic_msg) => {
                self.schematic.update(schematic_msg);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Msg> {
        let schematic = self.schematic.view().map(Msg::SchematicMsg);
        // let device_designer = self.designer.view().map(Msg::DeviceDesignerMsg);

        let tabs = Tabs::with_tabs(
            self.active_tab,
            vec![
                (TabLabel::Text("Schematic".to_string()), schematic),
                (
                    TabLabel::Text("Device Creator".to_string()),
                    // device_designer,
                    iced::widget::text("placeholder").into(),
                ),
            ],
            Msg::TabSel,
        );

        tabs.into()
    }
}

trait IcedStruct<T> {
    fn update(&mut self, msg: T);
    fn view(&self) -> Element<T>;
}
