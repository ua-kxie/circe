use bevy::{
    input::mouse::MouseWheel, pbr::{MaterialPipeline, MaterialPipelineKey}, prelude::*, reflect::TypePath, render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, SpecializedMeshPipelineError,
        },
    }, sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle}, window::PrimaryWindow
};

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
        layout: &MeshVertexBufferLayout,
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