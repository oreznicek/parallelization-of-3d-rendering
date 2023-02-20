
#[path = "../framework.rs"]
mod framework;
mod effects;
mod helper;

use wgpu::util::DeviceExt;
use std::path::Path;

struct Example {
    output_view: wgpu::TextureView,
    aspect_ratio_buf: wgpu::Buffer,
}

impl framework::Example for Example {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Example {

        // Create output texture
        let texture_path = Path::new("./examples/post-processing/original_scene.png");
        let texture_bytes_vec = std::fs::read(texture_path).unwrap();
        let texture_bytes = bytemuck::cast_slice(&texture_bytes_vec);

        let texture_image = image::load_from_memory(texture_bytes).unwrap();
        let texture_rgba = texture_image.to_rgba8();

        use image::GenericImageView;
        let (texture_width, texture_height) = texture_image.dimensions();

        let texture_extent = wgpu::Extent3d {
            width: texture_width,
            height: texture_height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let output_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        queue.write_texture(
            texture.as_image_copy(),
            &texture_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4*1024),
                rows_per_image: None,
            },
            texture_extent,
        );

        let aspect_ratio = config.width as f32 / config.height as f32;

        let aspect_ratio_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[aspect_ratio]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Example {
            output_view,
            aspect_ratio_buf,
        }
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let ratio = config.width as f32 / config.height as f32;
        queue.write_buffer(&self.aspect_ratio_buf, 0, bytemuck::cast_slice(&[ratio]));
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {
        // Empty
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _config: &wgpu::SurfaceConfiguration,
        _spawner: &framework::Spawner,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        effects::Tint::resolve(device, &mut encoder, &self.output_view, view, &self.aspect_ratio_buf, [1.0, 0.0, 0.0, 1.0]);
        queue.submit(Some(encoder.finish()));
    }
}

// Run the application
fn main() {
    framework::run::<Example>("Post processing");
}