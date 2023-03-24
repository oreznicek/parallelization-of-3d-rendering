
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv_coords: vec2<f32>
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>,
}

@group(0) @binding(0) var in_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

// Make grayscale from RGBA
fn grayscale(pixel: vec4<f32>) -> f32 {
    return (pixel.x + pixel.y + pixel.z) / 3.0;
}

// Makes kernel of surrounding pixels
fn make_kernel(uv: vec2<f32>) -> mat3x3<f32> {
    let dimensions = textureDimensions(in_texture);

    let w = 1.0 / f32(dimensions.x);
    let h = 1.0 / f32(dimensions.y);

    var kernel: mat3x3<f32>;

    kernel[0][0] = grayscale(textureSample(in_texture, texture_sampler, uv + vec2<f32>(-w, -h)));
    kernel[0][1] = grayscale(textureSample(in_texture, texture_sampler, uv + vec2<f32>(0.0, -h)));
    kernel[0][2] = grayscale(textureSample(in_texture, texture_sampler, uv + vec2<f32>(w, -h)));
    kernel[1][0] = grayscale(textureSample(in_texture, texture_sampler, uv + vec2<f32>(-w, 0.0)));
    kernel[1][1] = grayscale(textureSample(in_texture, texture_sampler, uv));
    kernel[1][2] = grayscale(textureSample(in_texture, texture_sampler, uv + vec2<f32>(w, 0.0)));
    kernel[2][0] = grayscale(textureSample(in_texture, texture_sampler, uv + vec2<f32>(-w, h)));
    kernel[2][1] = grayscale(textureSample(in_texture, texture_sampler, uv + vec2<f32>(0.0, h)));
    kernel[2][2] = grayscale(textureSample(in_texture, texture_sampler, uv + vec2<f32>(w, h)));

    return kernel;
}

fn make_sobel(pixels: mat3x3<f32>) -> f32 {
    // Define sobel filters
    let sobel_x: mat3x3<f32> = mat3x3<f32>(
         1.0,  0.0, -1.0,
         2.0,  0.0, -2.0,
         1.0,  0.0, -1.0,
    );

    let sobel_y: mat3x3<f32> = mat3x3<f32>(
         1.0,  2.0,  1.0,
         0.0,  0.0,  0.0,
        -1.0, -2.0, -1.0,
    );

    // Compute horizontal and vertical edge detection using Sobel operator
    let gx = dot(sobel_x[0], pixels[0]) + dot(sobel_x[1], pixels[1]) + dot(sobel_x[2], pixels[2]);
    let gy = dot(sobel_y[0], pixels[0]) + dot(sobel_y[1], pixels[1]) + dot(sobel_y[2], pixels[2]);

    // Compute gradient magnitude using Pythagorean theorem
    let g = sqrt(gx * gx + gy * gy);

    return g;
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv_coords = in.uv_coords;
    return out;
}

@fragment
fn fs_main(vert: VertexOutput) -> @location(0) vec4<f32> {
    var kernel = make_kernel(vert.uv_coords);
    var sobel = make_sobel(kernel);

    // Display edges with gradient magnitude greater than threshold
    /*let threshold = 0.3;
    if sobel > threshold { sobel = 1.0; }
    else { sobel = 0.0; }*/

    // Combine detected edges with the original image (additive blending)
    let out = textureSample(in_texture, texture_sampler, vert.uv_coords) + vec4<f32>(sobel, sobel, sobel, 0.0);

    return out;
}