// schematic tools - selection, drag move, etc.
mod sel;
pub mod wire;

const WIRE_TOOL_KEY: KeyCode = KeyCode::KeyW;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum SchematicToolState {
    #[default]
    Idle,
    Wiring,
    Grab,
    Label,   // wire/net labeling
    Comment, // plain text comment with basic formatting options
}

// different tools the schematic may have active

use bevy::prelude::*;

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(wire::WireToolPlugin);
        app.add_plugins(sel::SelToolPlugin);
        app.add_systems(Update, main);

        app.init_state::<SchematicToolState>();
    }
}

fn main(
    keys: Res<ButtonInput<KeyCode>>,
    curr_toolstate: Res<State<SchematicToolState>>,
    mut next_toolstate: ResMut<NextState<SchematicToolState>>,
) {
    if keys.just_released(KeyCode::Escape) {
        next_toolstate.set(SchematicToolState::Idle);
        return;
    }
    match curr_toolstate.get() {
        SchematicToolState::Idle => {
            if keys.just_released(WIRE_TOOL_KEY) {
                next_toolstate.set(SchematicToolState::Wiring);
            }
        }
        SchematicToolState::Grab => {}
        SchematicToolState::Wiring => {}
        SchematicToolState::Label => {}
        SchematicToolState::Comment => {}
    }
}
