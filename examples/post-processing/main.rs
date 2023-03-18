
#[path = "../framework.rs"]
mod framework;
mod effects;
mod helper;

use wgpu::util::DeviceExt;
use std::path::Path;
use std::borrow::Cow;
use effects::{ AllowedEffects, UVVertex, PostProcessing };
use helper::get_uv_from_position;

struct Example {
    vertex_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    post_processing: PostProcessing,
    output_view: Option<wgpu::TextureView>,
    aspect_ratio_buf: wgpu::Buffer,
}

impl framework::Example for Example {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Example {

        let vertices = [
            UVVertex { pos: [-1.0, -1.0], uv_coords: get_uv_from_position([-1.0, -1.0]) }, // 1
            UVVertex { pos: [ 1.0, -1.0], uv_coords: get_uv_from_position([ 1.0, -1.0]) }, // 2
            UVVertex { pos: [ 1.0,  1.0], uv_coords: get_uv_from_position([ 1.0,  1.0]) }, // 3
            UVVertex { pos: [ 1.0,  1.0], uv_coords: get_uv_from_position([ 1.0,  1.0]) }, // 3
            UVVertex { pos: [-1.0,  1.0], uv_coords: get_uv_from_position([-1.0,  1.0]) }, // 4
            UVVertex { pos: [-1.0, -1.0], uv_coords: get_uv_from_position([-1.0, -1.0]) }, // 1
        ];
        
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let aspect_ratio = config.width as f32 / config.height as f32;
        let aspect_ratio_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[aspect_ratio]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Previous frame texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Texture sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
               // Aspect ratio
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4),
                    },
                    count: None,
                }
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_size = std::mem::size_of::<UVVertex>();

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Vertex position
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                // UV coordinates
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 2,
                    shader_location: 1,
                },
            ],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create input texture
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
        let input_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

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

        // Create texture sampler
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: aspect_ratio_buf.as_entire_binding(),
                }
            ]
        });
        
        let output_view = effects::create_output_texture_view(device, config);

        let flags = AllowedEffects::CONTOUR;
        let post_processing = effects::PostProcessing::init(
            flags,
            device,
            queue,
            config,
            &output_view,
        );
        // Passnout final_frame do resolve metody

        println!("{} {}", flags.highest_bit(), AllowedEffects::CONTOUR.bits());

        Example {
            vertex_buf,
            pipeline,
            bind_group,
            post_processing,
            output_view: Some(output_view),
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

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: match &self.output_view {
                        Some(output_view) => output_view,
                        None => view
                    },
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,//Clear(wgpu::Color::BLACK),
                        store: true
                    }
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.draw(0..6, 0..1);
        }

        //effects::Tint::resolve(device, queue, &self.output_view, view, [1.0, 0.0, 0.0, 1.0]);
        //effects::Contour::resolve(device, &mut encoder, &self.output_view, view, &self.aspect_ration_buf);
        if let Some(output_view) = &self.output_view {
            self.post_processing.resolve(device, queue, view);
        }
        queue.submit(Some(encoder.finish()));
    }
}

// Run the application
fn main() {
    framework::run::<Example>("Post processing");
}