
struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>
}

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>
) -> VertexOutput {
    var result: VertexOutput;
    result.pos = position;
    result.color = color;
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color;
}
