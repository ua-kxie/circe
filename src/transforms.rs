//! types and constants facillitating geometry and transforms

use euclid::{Point2D, Transform2D};
use iced::Point as IcedPoint;
use serde::{Deserialize, Serialize};

/// PhantomData tag used to denote the patch of screen being drawn on (f32)
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize,
)]
pub struct CanvasSpace;

/// PhantomData tag used to denote the f32 space on which the schematic is drawn
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize,
)]
pub struct ViewportSpace;

/// PhantomData tag used to denote the i16 space in which the schematic exists
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize,
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
#[allow(dead_code)]
pub type SSVec = euclid::Vector2D<i16, SchematicSpace>;

/// viewport to canvas space transform
pub type VCTransform = euclid::Transform2D<f32, ViewportSpace, CanvasSpace>;
/// canvas to viewport space transform
pub type CVTransform = euclid::Transform2D<f32, CanvasSpace, ViewportSpace>;
/// schematic space transform
pub type SSTransform = euclid::Transform2D<i16, SchematicSpace, SchematicSpace>;

/// viewport to canvas space transform with locked aspect ratio
#[derive(Debug, Clone, Copy)]
pub struct VCTransformLockedAspect(VCTransform);
impl VCTransformLockedAspect {
    /// returns the identity transform of this type
    pub fn identity() -> Self {
        Self(VCTransform::identity())
    }
    /// flip transform along y-axis
    pub fn pre_flip_y(&self) -> Self {
        Self(self.0.pre_scale(1.0, -1.0))
    }
    /// get the scale factor of the transform
    pub fn scale(&self) -> f32 {
        self.0.m11.abs()
    }
    /// pre_translate
    pub fn pre_translate(&self, v: VSVec) -> Self {
        Self(self.0.pre_translate(v))
    }
    /// then_translate
    pub fn then_translate(&self, v: CSVec) -> Self {
        Self(self.0.then_translate(v))
    }
    /// then scale
    pub fn then_scale(&self, scale: f32) -> Self {
        Self(self.0.then_scale(scale, scale))
    }
    pub fn transform_point(&self, vsp: VSPoint) -> CSPoint {
        self.0.transform_point(vsp)
    }
    /// returns transform and scale such that VSBox (viewport/schematic bounds) fit inside CSBox (canvas bounds)
    pub fn fit_bounds(csb: CSBox, vsb: VSBox, min_zoom: f32, max_zoom: f32) -> Self {
        let mut vct = VCTransform::identity();

        let s = (csb.height() / vsb.height())
            .min(csb.width() / vsb.width())
            .clamp(min_zoom, max_zoom);
        vct = vct.then_scale(s, -s);
        // vector from vsb center to csb center
        let v = csb.center() - vct.transform_point(vsb.center());
        vct = vct.then_translate(v);

        Self(vct)
    }
    /// return the underlying transform
    pub fn transform(&self) -> VCTransform {
        self.0
    }
    /// return the inverse of the underlying transform
    pub fn inverse_transform(&self) -> CVTransform {
        self.0.inverse().unwrap()
    }
}

/// viewport to canvas space transform with independent x-y aspect ratios
#[derive(Debug, Clone, Copy)]
pub struct VCTransformFreeAspect(VCTransform);
impl VCTransformFreeAspect {
    /// returns the scale along the x scale
    pub fn x_scale(&self) -> f32 {
        self.0.m11.abs()
    }
    /// returns the scale along the y scale
    pub fn y_scale(&self) -> f32 {
        self.0.m22.abs()
    }
    /// returns the identity transform of this type
    pub fn identity() -> Self {
        Self(VCTransform::identity())
    }
    /// flip transform along y-axis
    pub fn pre_flip_y(&self) -> Self {
        Self(self.0.pre_scale(1.0, -1.0))
    }
    /// pre_translate
    pub fn pre_translate(&self, v: VSVec) -> Self {
        Self(self.0.pre_translate(v))
    }
    /// then_translate
    pub fn then_translate(&self, v: CSVec) -> Self {
        Self(self.0.then_translate(v))
    }
    /// then scale
    pub fn then_scale(&self, x_scale: f32, y_scale: f32) -> Self {
        Self(self.0.then_scale(x_scale, y_scale))
    }
    /// transform a point
    pub fn transform_point(&self, vsp: VSPoint) -> CSPoint {
        self.0.transform_point(vsp)
    }
    /// returns transform and scale such that VSBox (viewport/schematic bounds) fit inside CSBox (canvas bounds)
    pub fn fit_bounds(csb: CSBox, vsb: VSBox, min_zoom: f32, max_zoom: f32) -> Self {
        let mut vct = VCTransform::identity();

        let s = (csb.height() / vsb.height())
            .min(csb.width() / vsb.width())
            .clamp(min_zoom, max_zoom);
        vct = vct.then_scale(s, -s);
        // vector from vsb center to csb center
        let v = csb.center() - vct.transform_point(vsb.center());
        vct = vct.then_translate(v);

        Self(vct)
    }
    /// return the underlying transform
    pub fn transform(&self) -> VCTransform {
        self.0
    }
    /// return the inverse of the underlying transform
    pub fn inverse_transform(&self) -> CVTransform {
        self.0.inverse().unwrap()
    }
}

/// 90 deg clockwise rotation transform
pub const SST_CWR: SSTransform = SSTransform::new(0, -1, 1, 0, 0, 0);

/// 90 deg counter clockwise rotation transform
pub const SST_CCWR: SSTransform = SSTransform::new(0, 1, -1, 0, 0, 0);

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
