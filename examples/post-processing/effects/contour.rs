
use wgpu::util::DeviceExt;
use std::borrow::Cow;
use super::{UVVertex, EffectType};
use crate::get_uv_from_position;

// Edge detection using sobel operator to isolate the contours
pub struct Contour {
   vertex_buf: wgpu::Buffer,
   pipeline: wgpu::RenderPipeline,
   bind_group: wgpu::BindGroup,
}

impl super::Effect for Contour {
    fn init(
        device: &wgpu::Device,
        input_view: &wgpu::TextureView,
        _effect_type: EffectType,
    ) -> Contour {

        let vertices = [
            UVVertex { pos: [-1.0, -1.0], uv_coords: get_uv_from_position([-1.0, -1.0]) }, // 1
            UVVertex { pos: [ 1.0, -1.0], uv_coords: get_uv_from_position([ 1.0, -1.0]) }, // 2
            UVVertex { pos: [ 1.0,  1.0], uv_coords: get_uv_from_position([ 1.0,  1.0]) }, // 3
            UVVertex { pos: [ 1.0,  1.0], uv_coords: get_uv_from_position([ 1.0,  1.0]) }, // 3
            UVVertex { pos: [-1.0,  1.0], uv_coords: get_uv_from_position([-1.0,  1.0]) }, // 4
            UVVertex { pos: [-1.0, -1.0], uv_coords: get_uv_from_position([-1.0, -1.0]) }, // 1
        ];

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let contour_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("contour.wgsl"))),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Previous frame texture: texture_2d<f32>
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
                // Texture sampler: sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ]
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
                module: &contour_shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers
            },
            fragment: Some(wgpu::FragmentState {
                module: &contour_shader,
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
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ]
        });

        Contour {
            vertex_buf,
            pipeline,
            bind_group,
        }
    }

    fn resolve(
        &self, 
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_view: &wgpu::TextureView
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
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

        queue.submit(Some(encoder.finish()));
    }
}