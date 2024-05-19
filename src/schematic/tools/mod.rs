// schematic tools - selection, drag move, etc.
mod grab;
mod sel;
mod wire;

const WIRE_TOOL_KEY: KeyCode = KeyCode::KeyW;
const COPY_KEY: KeyCode = KeyCode::KeyC;
const MOVE_KEY: KeyCode = KeyCode::KeyM;

// marker components
#[derive(Component)]
struct Preview;

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Tentative;

// events
/// event fires when some system wants to clone a Wire entity
#[derive(Event)]
struct CloneToEnt(ElementType);

#[derive(Debug, Clone)]
enum ElementType {
    WireSeg((Entity, WireSeg)),
    // device
    // comment
    // etc.
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchematicToolState {
    #[default]
    Idle,
    Wiring,
    Grab,
    // Label,   // wire/net labeling
    // Comment, // plain text comment with basic formatting options
}

// different tools a schematic may have active

use bevy::prelude::*;

use self::wire::WireSeg;

use super::CursorMarker;

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            wire::WireToolPlugin,
            sel::SelToolPlugin,
            grab::GrabToolPlugin,
        ));
        app.add_systems(Update, main);
        app.add_systems(OnEnter(SchematicToolState::Idle), reset);

        app.init_state::<SchematicToolState>();
        app.add_event::<CloneToEnt>();
    }
}

fn main(
    keys: Res<ButtonInput<KeyCode>>,
    curr_toolstate: Res<State<SchematicToolState>>,
    mut next_toolstate: ResMut<NextState<SchematicToolState>>,
    q_selected: Query<&WireSeg, With<Selected>>,
    mut commands: Commands,
    mut e_clonetoent: EventWriter<CloneToEnt>,
    q_cursor: Query<Entity, With<CursorMarker>>,
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
            if keys.just_released(COPY_KEY) {
                let c = q_cursor.single();
                let mut entities = vec![];
                for ws in q_selected.iter() {
                    let ent = commands.spawn((ws.clone(), Preview)).id();
                    let et = ElementType::WireSeg((ent.clone(), ws.clone()));
                    e_clonetoent.send(CloneToEnt(et.clone()));
                    entities.push(ent);
                }
                commands.entity(c).push_children(&entities);
                next_toolstate.set(SchematicToolState::Grab);
            }
            if keys.just_released(MOVE_KEY) {
                next_toolstate.set(SchematicToolState::Grab);
            }
        }
        SchematicToolState::Grab => {}
        SchematicToolState::Wiring => {}
    }
}

fn reset(previews: Query<Entity, With<Preview>>, mut commands: Commands) {
    for e in &previews {
        commands.entity(e).despawn();
    }
}
