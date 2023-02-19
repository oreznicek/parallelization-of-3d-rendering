
use wgpu::util::{DeviceExt};
use std::borrow::Cow;
use bytemuck::{Pod, Zeroable};
use crate::helper::get_uv_from_position;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct UVVertex {
    pos: [f32; 2],
    uv_coords: [f32; 2]
}

// This effect will change the tone of the whole scene
// based on the input color
pub struct Tint {
    vertex_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl Tint {
    fn init(
        device: &wgpu::Device,
        input_view: &wgpu::TextureView,
        tint_color: [f32; 4],
    ) -> Tint {

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

        let tint_color_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&tint_color),
            usage: wgpu::BufferUsages::UNIFORM,
        });

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
                // Tint color -> vec4<f32>
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4 * 4),
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
                module: &tint_shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers
            },
            fragment: Some(wgpu::FragmentState {
                module: &tint_shader,
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: tint_color_buf.as_entire_binding(),
                },
            ]
        });

        Tint {
            vertex_buf,
            pipeline,
            bind_group,
        }
    }

    pub fn resolve(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        input_frame: &wgpu::TextureView,
        output_frame: &wgpu::TextureView,
        tint_color: [f32; 4],
    ) {
        let instance = Tint::init(device, input_frame, tint_color);

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_frame,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true
                    }
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&instance.pipeline);
            rpass.set_bind_group(0, &instance.bind_group, &[]);
            rpass.set_vertex_buffer(0, instance.vertex_buf.slice(..));
            rpass.draw(0..6, 0..1);
        }
    }
}