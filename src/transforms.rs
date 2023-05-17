use iced::Point as IcedPoint;
use euclid::{Point2D};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasSpace;  // viewport space tag - the patch of screen which is drawn on

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewportSpace;  // canvas space tag - the space in which the schematic exists in, in f32

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchematicSpace;  // schematic space tag - the space in which the schematic exists in, in i16

pub type CSPoint = euclid::Point2D<f32, CanvasSpace>;
pub type VSPoint = euclid::Point2D<f32, ViewportSpace>;
pub type SSPoint = euclid::Point2D<i16, SchematicSpace>;

pub type CSBox = euclid::Box2D<f32, CanvasSpace>;
pub type VSBox = euclid::Box2D<f32, ViewportSpace>;
pub type SSBox = euclid::Box2D<i16, SchematicSpace>;

pub type VCTransform = euclid::Transform2D<f32, ViewportSpace, CanvasSpace>;
pub type CVTransform = euclid::Transform2D<f32, CanvasSpace, ViewportSpace>;

/// Newtype for working with iced::Point and euclid::Point2D s
#[derive(Debug, Copy, Clone)]
pub struct Point(CSPoint);

impl From<IcedPoint> for Point {
    fn from(src: IcedPoint) -> Self {
        Point(Point2D::new(
            src.x,
            src.y
        ))
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