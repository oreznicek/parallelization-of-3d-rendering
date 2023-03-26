
fn abs(x: f32) -> f32 {
    if x < 0.0 {
        return x*(-1.0);
    }
    x
}

// This position equals [0, 0] in UV coordinates
const ORIGIN: [f32; 2] = [-1.0, 1.0];

// Calculate UV coordinates from position on the screen
pub fn get_uv_from_position(pos: [f32; 2]) -> [f32; 2] {
    let mut uv = [0.0; 2];

    uv[0] = abs(ORIGIN[0] - pos[0]) / 2.0;
    uv[1] = abs(ORIGIN[1] - pos[1]) / 2.0;

    uv
} 

// Creates output texture and returns its wgpu::TextureView
pub fn create_output_texture_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {

    let output_texture_extent = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };

    let output_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: output_texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    });
    let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

    output_view
}

// Creates specified number of texture views
pub fn create_output_texture_views(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    count: usize,
) -> Vec<wgpu::TextureView> {
    let mut output_views = Vec::new();

    for _i in 0..count {
        output_views.push(create_output_texture_view(device, config));
    }

    output_views
}