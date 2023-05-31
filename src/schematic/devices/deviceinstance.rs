use std::rc::Rc;

use super::devicetype::{Port, Graphics};

use euclid::{Size2D, Transform2D, Vector2D, Angle};
use iced::{widget::canvas::{Frame, Stroke, stroke, LineCap, path::Builder, self, LineDash}, Color, Size};

use crate::{
    schematic::nets::{Selectable, Drawable},
    transforms::{
        SSPoint, VSBox, SSBox, VSPoint, VCTransform, Point, ViewportSpace, SchematicSpace
    }, 
};


