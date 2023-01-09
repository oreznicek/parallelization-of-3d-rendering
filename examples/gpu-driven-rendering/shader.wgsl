
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
}

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@group(0)
@binding(1)
var<storage> matrices: array<mat4x4<f32>>;

@group(0)
@binding(2)
var texture_arr: binding_array<texture_2d<f32>>;

@group(0)
@binding(3)
var texture_sampler: sampler;

@group(1)
@binding(0)
var<uniform> texture_id: u32;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
    @builtin(instance_index) instance_id: u32,
) -> VertexOutput {
    var result: VertexOutput;
    result.tex_coord = tex_coord;
    result.position = transform * (matrices[instance_id] * position);
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_arr[texture_id], texture_sampler, vertex.tex_coord);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 0.5);
}
