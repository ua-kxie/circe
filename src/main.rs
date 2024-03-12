use bevy::prelude::*;

mod schematic;
mod types;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, schematic::SchematicPlugin))
        .run();
}
