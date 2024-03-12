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
use petgraph::graphmap::GraphMap;
use crate::types::SSPoint;

pub struct State {
    nets: Nets,
    devices: Devices,
    comments: Comments,
    labels: Labels,
    ports: Ports,
}

struct Nets{
    graph: Box<GraphMap<SSPoint, (), petgraph::Undirected>>,
}

struct Devices;
struct Comments;
struct Labels;
struct Ports;