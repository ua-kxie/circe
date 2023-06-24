use euclid::{Box2D, Point2D, Transform2D, Vector2D};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Canvas;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Viewport;

fn main() {
    let a = Point2D::<f32, Canvas>::new(1., 2.);
    let b = a.cast::<usize>();
}
