use bevy::{prelude::*, sprite::Material2dPlugin, window::PrimaryWindow};

mod schematic;
mod types;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            schematic::SchematicPlugin,
            MaterialPlugin::<schematic::grid::GridMaterial>::default(),
        ))
        .add_systems(Startup, hide_cursor)
        .run();
}
pub fn hide_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let _window = &mut primary_window.single_mut();
    // window.cursor.visible = false;
}
