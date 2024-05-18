// tool for making selections
use crate::{
    schematic::{NewCurposI, SchematicRes},
    types::{NewIVec2, SSBox},
};
use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        primitives::Aabb,
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

use super::SchematicToolState;


// events
// enter/exit grab tool can be done via tool state

// event for placing elements in preview
#[derive(Event)]
struct Place;

// systems
// system to run on entering tool
fn on_enter(
) {
    // handle from schematic, enter as copy, move, or placing of new element (e.g. device)
    // upon entering, provide with vector Entities which should be marked as preview and used to show transforms

    // on entering grab tool
    // make copy of all elements marked as selected, 
    // unmark as selected, 
    // mark all copied elements as preview
}

// system to run on exiting grab tool
fn on_exit(

) {
    // on exiting grab tool,
    // delete all elements marked as preview
}

// system to run on grab tool place
fn on_place(

) {
    // on place event:
    // make copy of all elements marked as preview
}

// main loop system to handle basic transformation commands
fn main(
) {
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
            (
                (
                    on_place,
                    main
                ).run_if(in_state(SchematicToolState::Grab)),
            ),
        );
        app.add_systems(OnEnter(SchematicToolState::Grab), on_enter);
        app.add_systems(OnExit(SchematicToolState::Grab), on_exit);
        app.add_event::<Place>();
    }
}