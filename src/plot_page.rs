//! Schematic GUI page
//! includes paramter editor, toolbar, and the canvas itself

use std::rc::Rc;

use crate::plot::{ChartElement, Msg as PlotMsg, Plot};
use crate::transforms::{VCTransformFreeAspect, VSPoint};
use crate::viewport_free_aspect::{CompositeMsg, Content, Msg};
use crate::{plot, viewport_free_aspect};

use crate::IcedStruct;
use iced::widget::row;
use iced::Element;

#[derive(Debug, Clone)]
pub enum PlotPageMsg {
    ViewportEvt(viewport_free_aspect::CompositeMsg<plot::Msg>),
    Traces(Vec<Vec<VSPoint>>),
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
            viewport: viewport_free_aspect::Viewport::new(1.0, f32::EPSILON, f32::MAX, vct),
        }
    }
}

impl IcedStruct<PlotPageMsg> for PlotPage {
    fn update(&mut self, msg: PlotPageMsg) {
        match msg {
            PlotPageMsg::ViewportEvt(msgs) => {
                self.viewport.update(msgs);
            }
            PlotPageMsg::Traces(traces) => {
                let content_msg = PlotMsg::Traces(traces);
                self.viewport.content.update(content_msg);
            }
        }
    }

    fn view(&self) -> Element<PlotPageMsg> {
        let str_ssp = format!(
            "curpos: x: {:.2e}; y: {:.2e}",
            self.viewport.curpos_vsp().x,
            self.viewport.curpos_vsp().y
        );
        let str_xyscales = format!(
            "scale: x: {:.2e}; y: {:.2e}",
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
