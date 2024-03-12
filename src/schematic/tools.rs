// different tools the schematic may have active

use bevy::{
    math::vec3,
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, SpecializedMeshPipelineError,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy::{
    input::mouse::MouseWheel, window::PrimaryWindow,
};

pub enum Tool {
    Wiring(Wiring),  
    Label,  // wire/net labeling
    Comment,  // plain text comment with basic formatting options
}

struct Wiring {
    mesh: Option<Handle<Mesh>>,


}