use std::marker::PhantomData;

use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, SpecializedMeshPipelineError,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle},
    window::PrimaryWindow,
};
use bevy::{
    render::render_resource::ShaderRef,
};

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(0)]
    pub(crate) color: Color,
}

impl Material2d for GridMaterial {
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = PolygonMode::Point;
        descriptor.primitive.topology = PrimitiveTopology::PointList;
        Ok(())
    }

    fn vertex_shader() -> ShaderRef {
        "grid_shader.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "grid_shader.wgsl".into()
    }
}