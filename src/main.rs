//! Circe
//! Schematic Capture for EDA with ngspice integration

use std::fmt::Debug;

mod analysis;
mod schematic;
mod transforms;

use analysis::plot_page::{PlotPage, PlotPageMsg};
use schematic::circuit_page::CircuitPage;
use schematic::symbols_page::DevicePage;

// mod designer;

use iced::{executor, Application, Command, Element, Settings, Theme};

use iced_aw::{TabLabel, Tabs};

pub fn main() -> iced::Result {
    Circe::run(Settings {
        window: iced::window::Settings {
            size: (800, 500),
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
    plot_view: PlotPage,

    /// active tab index
    active_tab: usize,
    designer: DevicePage,
}

#[derive(Debug, Clone)]
pub enum Msg {
    DesignerMsg(schematic::symbols_page::DevicePageMsg),
    SchematicMsg(schematic::circuit_page::CircuitPageMsg),
    PlotViewMsg(analysis::plot_page::PlotPageMsg),
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
                designer: DevicePage::default(),
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
            Msg::DesignerMsg(device_designer_msg) => {
                self.designer.update(device_designer_msg);
            }
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
        let devices = self.designer.view().map(Msg::DesignerMsg);

        let tabs = Tabs::with_tabs(
            vec![
                (0, TabLabel::Text("Graphs".to_string()), plot),
                (1, TabLabel::Text("Schematic".to_string()), schematic),
                (2, TabLabel::Text("Device Designer".to_string()), devices),
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
use crate::transforms::VCTransform;
use iced::widget::canvas::Frame;
/// trait for element which can be drawn on canvas
pub trait Drawable {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame);
}
