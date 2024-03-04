use bevy::prelude::*;

mod schematic;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, schematic::SchematicPlugin))
        .run();
}
