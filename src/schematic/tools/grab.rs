// tool for making selections

use bevy::prelude::*;

use crate::schematic::SchematicRes;

use super::{wire::WireSeg, CloneToEnt, ElementType, Preview, SchematicToolState};

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
    // pes: Res<PreviewElements>,
    q_pes: Query<(&GlobalTransform, &WireSeg), With<Preview>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut e_clonetoent: EventWriter<CloneToEnt>,
    mut commands: Commands,
) {
    // on place event:
    // make copy of all elements marked as preview

    // whats a better way for grab tool to branch off based on different type?
    // systemId callback - create a clone callback in each type
    // get callback here, call it with a transform arg - but wire segs have 2 points that can have different relative distances...
    if schematic_res.cursor_position.opt_ssp.is_some() && buttons.just_pressed(PLACE_BUTTON) {
        for (gt, ws) in q_pes.iter() {
            let ent = commands.spawn(ws.clone()).id();
            let ws1 = WireSeg::new(
                gt.transform_point(ws.p0().extend(0).as_vec3())
                    .as_ivec3()
                    .truncate(),
                gt.transform_point(ws.p1().extend(0).as_vec3())
                    .as_ivec3()
                    .truncate(),
            );
            e_clonetoent.send(CloneToEnt(ElementType::WireSeg((ent.clone(), ws1.clone()))));
            // match pe {
            //     ElementType::WireSeg((e, ws)) => {
            //         let ent = commands.spawn(ws.clone()).id();
            //         e_clonetoent.send(CloneToEnt(ElementType::WireSeg((ent.clone(), ws.clone()))));
            //     },
            // }
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
