pub mod wire;

const WIRE_TOOL_KEY: KeyCode = KeyCode::KeyE;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum SchematicToolState {
    #[default]
    Idle,
    Wiring,
    Label,   // wire/net labeling
    Comment, // plain text comment with basic formatting options
}

// different tools the schematic may have active

use bevy::{prelude::*, ui::update};

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


pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(wire::Wire);
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
        return
    }
    match curr_toolstate.get() {
        SchematicToolState::Idle => {
            if keys.just_released(WIRE_TOOL_KEY) {
                next_toolstate.set(SchematicToolState::Wiring);
            }
        },
        SchematicToolState::Wiring => todo!(),
        SchematicToolState::Label => todo!(),
        SchematicToolState::Comment => todo!(),
    }
}