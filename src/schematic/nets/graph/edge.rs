/// NetEdge - Edge Weight used in petgraph::Graph

use std::rc::Rc;

use super::SchematicNetLabel;
use crate::{
    transforms::{
        SSPoint, VCTransform, SSBox, SSTransform
    }, 
    schematic::{interactable::{Interactable, Interactive}, nets::Drawable}
};

use iced::{widget::canvas::{Frame, Path, Stroke, stroke, LineCap, LineDash}, Color};

/// A NetEdge represents a segment of wiring. 
/// It exists in the program as an edge weight for petgraph::Graph. 
#[derive(Clone, Debug, Default)]
pub struct NetEdge {
    /// source point of edge segment
    pub src: SSPoint,
    /// destination point of edge segment
    pub dst: SSPoint,
    /// interactable associated with this edge segment
    pub interactable: Interactable,
    /// auto generated net name associated with this edge segment
    pub label: Option<Rc<String>>,
    /// user defined net name assigned to this edge segment
    pub schematic_net_label: Option<SchematicNetLabel>,
}

/// two edges are equal if their source and destination pts are equal
impl PartialEq for NetEdge {
    fn eq(&self, other: &Self) -> bool {
        self.src == other.src && self.dst == other.dst
    }
}

/// hash absed on the soruce and destination points
impl std::hash::Hash for NetEdge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.src.hash(state);
        self.dst.hash(state);
    }
}

impl NetEdge {
    /// creates an interactable based on source and destination points, with settable 'tentative' flag
    pub fn interactable(src: SSPoint, dst: SSPoint, tentative: bool) -> Interactable {
        Interactable { bounds: NetEdge::bounds_from_pts(src, dst), tentative, }
    }
    /// creates a bound based on source and destination points - return value is guaranteed to have positive area
    pub fn bounds_from_pts(src: SSPoint, dst: SSPoint) -> SSBox {
        SSBox::from_points([src, dst])
    }
    /// checks if argument SSPoint lies on the edge (excludes source and destination points)
    pub fn intersects_ssp(&self, ssp: SSPoint) -> bool {
        self.interactable.contains_ssp(ssp) && self.src != ssp && self.dst != ssp
    }
}

impl Interactive for NetEdge {
    /// transform the edge based on SSTransform argument
    fn transform(&mut self, sst: SSTransform) {
        (self.src, self.dst) = (
            sst.transform_point(self.src),
            sst.transform_point(self.dst),
        );
        self.interactable.bounds = NetEdge::bounds_from_pts(self.src, self.dst);
    }
}

/// helper function for drawing the netedge on the canvas
fn draw_with(src: SSPoint, dst: SSPoint, vct: VCTransform, frame: &mut Frame, stroke: Stroke) {
    let psrcv = vct.transform_point(src.cast().cast_unit());
    let pdstv = vct.transform_point(dst.cast().cast_unit());
    let c = Path::line(
        iced::Point::from([psrcv.x, psrcv.y]),
        iced::Point::from([pdstv.x, pdstv.y]),
    );
    frame.stroke(&c, stroke);
}

/// width of the wire segment
const WIRE_WIDTH: f32 = 0.05;
/// zoom level below which wire width stops becoming thinner
const ZOOM_THRESHOLD: f32 = 5.0;

impl Drawable for NetEdge {
    fn draw_persistent(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let wire_width = self::WIRE_WIDTH;
        let zoom_thshld = self::ZOOM_THRESHOLD;
        let wire_stroke = Stroke {
            width: (wire_width * vcscale).max(wire_width * zoom_thshld),
            style: stroke::Style::Solid(Color::from_rgb(0.0, 0.8, 1.0)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        draw_with(self.src, self.dst, vct, frame, wire_stroke);
    }
    fn draw_selected(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let wire_width = self::WIRE_WIDTH;
        let zoom_thshld = self::ZOOM_THRESHOLD;
        let wire_stroke = Stroke {
            width: (wire_width * vcscale).max(wire_width * zoom_thshld),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 0.8, 0.0)),
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        draw_with(self.src, self.dst, vct, frame, wire_stroke);
    }
    fn draw_preview(&self, vct: VCTransform, vcscale: f32, frame: &mut Frame) {
        let wire_width = self::WIRE_WIDTH;
        let zoom_thshld = self::ZOOM_THRESHOLD;
        let wire_stroke = Stroke {
            width: (wire_width * vcscale).max(wire_width * zoom_thshld),
            style: stroke::Style::Solid(Color::from_rgb(1.0, 1.0, 0.5)),
            line_cap: LineCap::Butt,
            line_dash: LineDash{segments: &[3. * (wire_width * vcscale).max(wire_width * 2.0)], offset: 0},
            ..Stroke::default()
        };
        draw_with(self.src, self.dst, vct, frame, wire_stroke);
    }
}