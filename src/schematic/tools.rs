// different tools the schematic may have active

use bevy::prelude::*;

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
    pub mesh: Option<Handle<Mesh>>,
}
