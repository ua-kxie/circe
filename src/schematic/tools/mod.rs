// schematic tools - selection, drag move, etc.
mod sel;
mod wire;
mod grab;

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
struct CloneToEnt((Entity, ElementType));

enum ElementType {
    WireSeg(WireSeg),
    // device
    // comment
    // etc.
}

// resources
// previewElements: vector of elements used for preview (e.g. for grab tool)
#[derive(Resource, Default)]
struct PreviewElements {
    ve: Vec<Entity>,  // stores all the entities marked as preview
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchematicToolState {
    #[default]
    Idle,
    Wiring,
    Grab,
    Label,   // wire/net labeling
    Comment, // plain text comment with basic formatting options
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
        app.init_resource::<PreviewElements>();
        app.add_event::<CloneToEnt>();
    }
}

fn main(
    keys: Res<ButtonInput<KeyCode>>,
    curr_toolstate: Res<State<SchematicToolState>>,
    mut next_toolstate: ResMut<NextState<SchematicToolState>>,
    q_selected: Query<&wire::WireSeg, With<Selected>>,
    mut commands: Commands,
    mut e_clonetoent: EventWriter<CloneToEnt>,
    mut previews: ResMut<PreviewElements>,
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
                previews.ve.clear();
                let c = q_cursor.single();
                for ws in q_selected.iter() {
                    let ent = commands.spawn((ws.clone(), Preview)).id();
                    e_clonetoent.send(CloneToEnt((
                        ent.clone(), ElementType::WireSeg(ws.clone())
                    )));
                    previews.ve.push(ent);
                }
                commands.entity(c).push_children(&previews.ve);
                next_toolstate.set(SchematicToolState::Grab);
            }
            if keys.just_released(MOVE_KEY) {
                next_toolstate.set(SchematicToolState::Grab);
            }
        }
        SchematicToolState::Grab => {}
        SchematicToolState::Wiring => {}
        SchematicToolState::Label => {}
        SchematicToolState::Comment => {}
    }
}

fn reset (
    mut previews: ResMut<PreviewElements>,
    mut commands: Commands,
) {
    for e in &previews.ve {
        commands.entity(*e).despawn();
    }
    previews.ve.clear();
}