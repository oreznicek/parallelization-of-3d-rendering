
use wgpu::util::{DeviceExt};
use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct TintVertex {
    uv_coords: [f32; 2]
}

// This effect will change the tone of the whole scene
// based on the input color
pub struct Tint<'a> {
    vertex_buf: &'a wgpu::Buffer,
    vertex_count: u32,
    pipeline: wgpu::RenderPipeline,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    output_view: &'a wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

impl<'a> Tint<'a> {
    // New will create pipeline for this specific post-processing effect
    // With all the buffer bindings needed
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_view: &wgpu::TextureView,
        config: &wgpu::SurfaceConfiguration,
        vertex_buf: &wgpu::Buffer,
        vertex_count: u32,
        tint_color: [f32; 4]
    ) -> Self {

        // Create tint color uniform
        let tint_color_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&tint_color),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // Create shader module
        let tint_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("tint.wgsl"))),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Previous frame texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Texture sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Tint color -> vec2<f32>
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4 * 2),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_size = std::mem::size_of::<TintVertex>();

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // UV coordinates
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
            ],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &tint_shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers
            },
            fragment: Some(wgpu::FragmentState {
                module: &tint_shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())]
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

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
                    resource: tint_color_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&output_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ]
        });

        Self {
            vertex_buf,
            vertex_count,
            pipeline,
            device,
            queue,
            output_view,
            bind_group,
        }
    }

    // Resolve frame will take the output frame of previous rendering as an input
    // It will apply the effect to the scene
    pub fn resolve_frame(self) {
        std::mem::drop(self);
    }
}

impl<'a> Drop for Tint<'a> {
    fn drop(&mut self) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true
                    }
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.draw(0..self.vertex_count, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
    }
}