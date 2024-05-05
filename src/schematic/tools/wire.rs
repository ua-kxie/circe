
pub struct Wire;
use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, SpecializedMeshPipelineError,
        },
    },
};

#[derive(Default)]
enum WiringToolStates {
    #[default]
    Ready,  // ready to place first anchor
    Drawing,  // placing second anchor point
}

use crate::types::SSPoint;

#[derive(Component)]
struct WireSeg {
    p0: SSPoint,
    p1: SSPoint,
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct WireMaterial {
    #[uniform(0)]
    pub(crate) color: Color,
}

impl Material for WireMaterial {
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

#[derive(Bundle)]
struct WireSegBundle {
    wire_seg: WireSeg,
    mesh: MaterialMeshBundle<WireMaterial>,
}


impl Plugin for Wire {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, main);
    }
}

fn setup() {}

fn main() {}