// different tools the schematic may have active

use bevy::prelude::*;

pub enum Tool {
    Wiring(Wiring),
    Label,   // wire/net labeling
    Comment, // plain text comment with basic formatting options
}

struct Wiring {
    mesh: Option<Handle<Mesh>>,
}
