//! types and constants facillitating geometry and transforms

use euclid::{Point2D, Transform2D};
use iced::Point as IcedPoint;

/// PhantomData tag used to denote the patch of screen being drawn on
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct CanvasSpace;

/// PhantomData tag used to denote the f32 space on which the schematic is drawn
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct ViewportSpace;

/// PhantomData tag used to denote the i16 space in which the schematic exists
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct SchematicSpace;

/// CanvasSpace Point
pub type CSPoint = euclid::Point2D<f32, CanvasSpace>;
/// ViewportSpace Point
pub type VSPoint = euclid::Point2D<f32, ViewportSpace>;
/// SchematicSpace Point
pub type SSPoint = euclid::Point2D<i16, SchematicSpace>;

/// CanvasSpace Box
pub type CSBox = euclid::Box2D<f32, CanvasSpace>;
/// ViewportSpace Box
pub type VSBox = euclid::Box2D<f32, ViewportSpace>;
/// SchematicSpace Box
pub type SSBox = euclid::Box2D<i16, SchematicSpace>;

/// CanvasSpace Vector
pub type CSVec = euclid::Vector2D<f32, CanvasSpace>;
/// ViewportSpace Vector
pub type VSVec = euclid::Vector2D<f32, ViewportSpace>;
/// SchematicSpace Vector
pub type SSVec = euclid::Vector2D<i16, SchematicSpace>;

/// viewport to canvas space transform
pub type VCTransform = euclid::Transform2D<f32, ViewportSpace, CanvasSpace>;
/// canvas to viewport space transform
pub type CVTransform = euclid::Transform2D<f32, CanvasSpace, ViewportSpace>;
/// schematic space transform
pub type SSTransform = euclid::Transform2D<i16, SchematicSpace, SchematicSpace>;

/// 90 deg clockwise rotation transform
pub const SST_CWR: SSTransform = SSTransform::new(0, -1, 1, 0, 0, 0);

/// 90 deg counter clockwise rotation transform
pub const SST_CCWR: SSTransform = SSTransform::new(0, 1, -1, 0, 0, 0);

///
pub const DESIGNER_GRID: f32 = 0.25;

/// 90 deg counter clockwise rotation transform
pub fn designer_ssp_to_schematic_vsp(ssp: SSPoint) -> VSPoint {
    ssp.cast::<f32>().cast_unit() * self::DESIGNER_GRID
}

/// converts SSTransform to VVTransform so that it can be composited with VCTransform
pub fn sst_to_xxt<T>(sst: SSTransform) -> Transform2D<f32, T, T> {
    sst.cast().with_destination().with_source()
}

/// Newtype for working with iced::Point and euclid::Point2D s
#[derive(Debug, Copy, Clone)]
pub struct Point(CSPoint);

impl From<IcedPoint> for Point {
    fn from(src: IcedPoint) -> Self {
        Point(Point2D::new(src.x, src.y))
    }
}

impl From<Point> for IcedPoint {
    fn from(src: Point) -> Self {
        IcedPoint::new(src.0.x, src.0.y)
    }
}

impl From<Point> for CSPoint {
    fn from(src: Point) -> Self {
        src.0
    }
}

impl From<CSPoint> for Point {
    fn from(src: CSPoint) -> Self {
        Self(src)
    }
}
