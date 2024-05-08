// general purpose shader to minimize pipeline count - wip

#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

struct SchematicMaterial {
    color: vec4<f32>,
    depth: f32,
    // color_neg: vec4<f32>,
};
@group(2) @binding(0) var<uniform> material: SchematicMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        get_model_matrix(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );
    // out.clip_position[2] = 1.0;  // keep the z coordinate fixed to maintain rendered size of lines and points
    return out;
}

@fragment
// fn fragment(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return material.color_pos;
    // return vec4(coord.x/1920.0, coord.y/1080.0, 1.0, 1.0);
    // return vec4(1.0, input.clip_position.x/1920.0, input.clip_position.y/1080.0, 0.2);
}