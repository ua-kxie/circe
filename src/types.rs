//! types and constants facillitating geometry and transforms

use bevy::math::{IVec2, Vec3};
use serde::{Deserialize, Serialize};

/// PhantomData tag used to denote the i16 space in which the schematic exists
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize,
)]
pub struct SchematicSpace;
/// PhantomData tag used to denote the patch of screen being drawn on (f32)
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize,
)]
pub struct CanvasSpace;

/// SchematicSpace Point
pub type SSPoint = euclid::Point2D<i32, SchematicSpace>;

/// CanvasSpace Point
pub type CSPoint = euclid::Point2D<f32, CanvasSpace>;

/// SchematicSpace Box
pub type SSBox = euclid::Box2D<i32, SchematicSpace>;

/// Newtype for Point2D<i32, SchematicSpace> <-> IVec2
#[derive(Debug, Copy, Clone)]
pub struct NewIVec2(IVec2);

impl From<IVec2> for NewIVec2 {
    fn from(src: IVec2) -> Self {
        NewIVec2(src)
    }
}

impl From<NewIVec2> for IVec2 {
    fn from(src: NewIVec2) -> Self {
        src.0
    }
}

impl From<SSPoint> for NewIVec2 {
    fn from(src: SSPoint) -> Self {
        NewIVec2(IVec2 { x: src.x, y: src.y })
    }
}

impl From<NewIVec2> for SSPoint {
    fn from(src: NewIVec2) -> Self {
        SSPoint::new(src.0.x, src.0.y)
    }
}

/// Newtype for working with bevy::Vec3 and euclid::Point2D s
#[derive(Debug, Copy, Clone)]
pub struct Point(CSPoint);

impl From<Vec3> for Point {
    fn from(src: Vec3) -> Self {
        Point(CSPoint::new(src.x, src.y))
    }
}

impl From<Point> for Vec3 {
    fn from(src: Point) -> Self {
        Vec3::new(src.0.x, src.0.y, 0.0)
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
