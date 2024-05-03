pub mod wire;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum SchematicToolState {
    #[default]
    Idle,
    Wiring,
}

// different tools the schematic may have active

use bevy::prelude::*;

use crate::types::SSPoint;

#[derive(Default)]
pub enum ActiveTool {
    #[default]
    Idle,
    Wiring(Box<Wiring>),
    Label,   // wire/net labeling
    Comment, // plain text comment with basic formatting options
}

#[derive(Default)]
pub struct Wiring {
    pub mesh: Option<(SSPoint, Handle<Mesh>, Entity)>,
}
