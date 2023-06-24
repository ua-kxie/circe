use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

struct NetEdge;
struct R;
struct Gnd;
struct Port;
struct Graphics<T> {
    pts: usize,
    bounds: usize,
    ports: Vec<Port>,
    marker: PhantomData<T>,
}
pub struct SingleValue<T> {
    value: f32,
    marker: core::marker::PhantomData<T>,
}
impl<T> SingleValue<T> {
    fn new() -> Self {
        SingleValue {
            value: 0.0,
            marker: core::marker::PhantomData,
        }
    }
}
pub enum Param<T> {
    Value(SingleValue<T>),
}
struct DeviceType<T> {
    graphics: Rc<Graphics<T>>,
    params: Param<T>,
}
enum DeviceEnum {
    R(DeviceType<R>),
    Gnd(DeviceType<Gnd>),
}
struct Device {
    common: usize,
    specific: DeviceEnum,
}

enum BaseElement {
    WireSeg(NetEdge),
    Device(Rc<RefCell<Device>>),
}

fn main() {}
