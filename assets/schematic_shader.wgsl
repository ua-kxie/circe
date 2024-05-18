// general purpose shader to minimize pipeline count - wip
// color: base color of the schematic element
// selected: bool flag of whether material belongs to selected element
// tentative: bool flag of whether material belongs to tentative element

#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var<uniform> selected: f32;
@group(2) @binding(2) var<uniform> tentative: f32;
@group(2) @binding(3) var<uniform> preview: f32;

const SELECTED_COLOR = vec4(2.0, 1.0, 0.0, 1.0) * 0.3;
const TENTATIVE_COLOR = vec4(1.0, 1.0, 1.0, 1.0);
const PREVIEW_COLOR = vec4(1.0, 1.0, 1.0, 0.2);

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
    return out;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return 
    (color + f32(selected) * SELECTED_COLOR + f32(tentative) * TENTATIVE_COLOR) 
    * max(PREVIEW_COLOR, vec4<f32>(preview,preview,preview,preview));
    // return material.color;
}
