
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) texture_id: u32,
}

struct Object {
    transform_id: u32,
    texture_id: u32,
}

@group(0) @binding(0) var<uniform> transform: mat4x4<f32>;
@group(0) @binding(1) var<storage> matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage> objects: array<Object>;
@group(0) @binding(3) var texture_arr: binding_array<texture_2d<f32>>;
@group(0) @binding(4) var texture_sampler: sampler;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
    @builtin(instance_index) instance_id: u32,
) -> VertexOutput {
    var result: VertexOutput;
    let object: Object = objects[instance_id];
    result.position = transform * (matrices[object.transform_id] * position);
    result.tex_coord = tex_coord;
    result.texture_id = object.texture_id;
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_arr[vertex.texture_id], texture_sampler, vertex.tex_coord);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 0.5);
}
