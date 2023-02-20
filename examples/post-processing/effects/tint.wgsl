
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv_coords: vec2<f32>
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>
}

@group(0)
@binding(0)
var in_texture: texture_2d<f32>;

@group(0)
@binding(1)
var texture_sampler: sampler;

@group(0)
@binding(2)
var<uniform> tint_color: vec4<f32>;

@group(0)
@binding(3)
var<uniform> aspect_ratio: f32;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.position * vec2<f32>(1.0 / aspect_ratio, 1.0), 0.0, 1.0);
    out.uv_coords = in.uv_coords;
    return out;
}

@fragment
fn fs_main(vert: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(in_texture, texture_sampler, vert.uv_coords) * tint_color;
}