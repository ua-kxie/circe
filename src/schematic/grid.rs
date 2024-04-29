// use bevy::{
//     prelude::*,
//     reflect::TypePath,
//     render::{
//         mesh::{MeshVertexBufferLayout, PrimitiveTopology},
//         render_resource::{
//             AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
//             SpecializedMeshPipelineError,
//         },
//     },
//     sprite::{Material2d, Material2dKey},
// };


use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexFormat,
        },
    },
};

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
}

impl Material for GridMaterial {
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            // ATTRIBUTE_BLEND_COLOR.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
    // fn specialize(
    //     descriptor: &mut RenderPipelineDescriptor,
    //     layout: &MeshVertexBufferLayout,
    //     _key: Material2dKey<Self>,
    // ) -> Result<(), SpecializedMeshPipelineError> {
    //     // descriptor.primitive.polygon_mode = PolygonMode::Point;
    //     // descriptor.primitive.topology = PrimitiveTopology::PointList;
    //     let vertex_layout = layout.get_layout(&[
    //         Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
    //         // ATTRIBUTE_BLEND_COLOR.at_shader_location(1),
    //     ])?;
    //     descriptor.vertex.buffers = vec![vertex_layout];
    //     Ok(())
    // }

    fn vertex_shader() -> ShaderRef {
        "grid_shader.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "grid_shader.wgsl".into()
    }
}
