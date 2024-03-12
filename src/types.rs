//! types and constants facillitating geometry and transforms

use serde::{Deserialize, Serialize};

/// PhantomData tag used to denote the i16 space in which the schematic exists
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize,
)]
pub struct SchematicSpace;

/// SchematicSpace Point
pub type SSPoint = euclid::Point2D<i16, SchematicSpace>;
