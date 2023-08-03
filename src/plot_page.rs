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
        let str_xyscales = format!(
            "x: {:04.1}; y: {:04.1}",
            self.viewport.vct().x_scale(),
            self.viewport.vct().y_scale(),
        );

        let canvas = self.viewport.view().map(PlotPageMsg::ViewportEvt);
        let infobar = row![
            iced::widget::text(str_ssp)
                .size(16)
                .height(16)
                .vertical_alignment(iced::alignment::Vertical::Center),
            iced::widget::text(str_xyscales)
                .size(16)
                .height(16)
                .vertical_alignment(iced::alignment::Vertical::Center),
        ]
        .spacing(10);

        let schematic = iced::widget::column![canvas, infobar,];

        schematic.into()
    }
}
