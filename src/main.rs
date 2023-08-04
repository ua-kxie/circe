//! Circe
//! Schematic Capture for EDA with ngspice integration

use std::{fmt::Debug, rc::Rc};

mod transforms;
// use designer::DeviceDesigner;
mod viewport;
mod viewport_free_aspect;

mod circuit;
mod circuit_gui;
mod plot;
mod plot_page;
mod schematic;

use circuit_gui::CircuitPage;
use plot_page::{PlotPage, PlotPageMsg};

// mod designer;

use iced::{executor, Application, Command, Element, Settings, Theme};

use iced_aw::{TabLabel, Tabs};
use transforms::{Point, VSPoint};

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
    schematic: CircuitPage,
    /// intended for dev use for now, can be recycled for user use to design subcircuit (.model) devices
    // designer: DeviceDesigner,
    plot_view: PlotPage,

    /// active tab index
    active_tab: usize,
}

#[derive(Debug, Clone)]
pub enum Msg {
    // DeviceDesignerMsg(designer::DeviceDesignerMsg),
    SchematicMsg(circuit_gui::CircuitPageMsg),
    PlotViewMsg(plot_page::PlotPageMsg),
    // Event(Event),
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
                schematic: CircuitPage::default(),
                // designer: DeviceDesigner::default(),
                plot_view: PlotPage::default(),
                active_tab: 1,
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

                if let Some(traces) = self.schematic.traces.take() {
                    let msg = PlotPageMsg::Traces(traces);
                    self.plot_view.update(msg);
                }
            }
            // Msg::DeviceDesignerMsg(device_designer_msg) => {
            //     self.designer.update(device_designer_msg);
            // }
            Msg::PlotViewMsg(plot_msg) => {
                self.plot_view.update(plot_msg);
            }
            Msg::SchematicMsg(schematic_msg) => {
                self.schematic.update(schematic_msg);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Msg> {
        let schematic = self.schematic.view().map(Msg::SchematicMsg);
        let plot = self.plot_view.view().map(Msg::PlotViewMsg);
        // let device_designer = self.designer.view().map(Msg::DeviceDesignerMsg);

        let tabs = Tabs::with_tabs(
            vec![
                (0, TabLabel::Text("Graphs".to_string()), plot),
                (1, TabLabel::Text("Schematic".to_string()), schematic),
                (
                    2,
                    TabLabel::Text("Device Creator".to_string()),
                    // device_designer,
                    iced::widget::text("placeholder").into(),
                ),
            ],
            Msg::TabSel,
        );

        tabs.set_active_tab(&self.active_tab).into()
    }
}

trait IcedStruct<T> {
    fn update(&mut self, msg: T);
    fn view(&self) -> Element<T>;
}
