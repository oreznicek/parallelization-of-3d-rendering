
#[path = "../framework.rs"]
mod framework;

use wgpu::util::DeviceExt;
use std::borrow::Cow;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4]
}

fn vertex(p: [f32; 2], c: [f32; 3]) -> Vertex {
    Vertex {
        pos: [p[0], p[1], 0.0, 1.0],
        color: [c[0], c[1], c[2], 1.0]
    }
}

struct Example {
    vertex_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl framework::Example for Example {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Example {

        // vertex0_position, vertex0_color, vertex1_position, ...
        let vertices = [
            vertex([ 0.0,  0.5], [1.0, 0.0, 0.0]),
            vertex([-0.5, -0.5], [0.0, 1.0, 0.0]),
            vertex([ 0.5, -0.5], [0.0, 0.0, 1.0])
        ];

        // Create vertex buffer
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX
        });

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let vertex_size = std::mem::size_of::<Vertex>();

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())]
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Example {
            vertex_buf,
            pipeline,
        }
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        // Empty
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {
        // Empty
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spawner: &framework::Spawner,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true
                    },
                })],
                depth_stencil_attachment: None
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}

// Run the application
fn main() {
    framework::run::<Example>("Triangle");
}