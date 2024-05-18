// tool for making selections

use bevy::prelude::*;

use crate::schematic::{self, SchematicRes};

use super::{CloneToEnt, ElementType, PreviewElements, SchematicToolState};

const PLACE_BUTTON: MouseButton = MouseButton::Left;

// events
// enter/exit grab tool can be done via tool state

// event for placing elements in preview
#[derive(Event)]
struct Place;

// systems
// system to run on entering tool
fn on_enter() {
    // handle from schematic, enter as copy, move, or placing of new element (e.g. device)

    // on entering grab tool
    // make copy of all elements marked as selected,
    // unmark as selected,
    // mark all copied elements as preview
}

// system to run on exiting grab tool
fn on_exit() {
    // on exiting grab tool,
    // delete all elements marked as preview
}

// system to run on grab tool 'place' event
fn on_place(
    schematic_res: Res<SchematicRes>,
    pes: Res<PreviewElements>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut e_clonetoent: EventWriter<CloneToEnt>,
    mut commands: Commands,
) {
    // on place event:
    // make copy of all elements marked as preview
    if schematic_res.cursor_position.opt_ssp.is_some() && buttons.just_pressed(PLACE_BUTTON) {
        for pe in pes.ve.iter() {
            match pe {
                ElementType::WireSeg((e, ws)) => {
                    let ent = commands.spawn(ws.clone()).id();
                    e_clonetoent.send(CloneToEnt(ElementType::WireSeg((ent.clone(), ws.clone()))));
                },
            }
        }
    }
}

// main loop system to handle basic transformation commands
fn main() {
    // transforms to handle:
    // translation: mouse movement
    // rotation cw ccw
    // mirror x/y
}

pub struct GrabToolPlugin;

impl Plugin for GrabToolPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, (setup,));
        app.add_systems(
            Update,
            ((on_place, main).run_if(in_state(SchematicToolState::Grab)),),
        );
        app.add_systems(OnEnter(SchematicToolState::Grab), on_enter);
        app.add_systems(OnExit(SchematicToolState::Grab), on_exit);
        app.add_event::<Place>();
    }
}
