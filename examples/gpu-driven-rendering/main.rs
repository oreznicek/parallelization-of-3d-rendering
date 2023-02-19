#[path = "../framework.rs"]
mod framework;
mod shapes;

use shapes::{Object, Batch, Mesh, MeshType, TextureType, TEXTURE_TYPE_VARIANTS};
use std::{borrow::Cow, f32::consts, mem, vec::Vec};
use std::path::Path;
use wgpu::util::DeviceExt;

struct Example {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    texture_id_count: Vec<u32>,
    bind_group0: wgpu::BindGroup,
    bind_groups1: Vec<wgpu::BindGroup>,
    uniform_buf: wgpu::Buffer,
    indirect_bufs: Vec<wgpu::Buffer>,
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
}

impl Example {
    fn generate_matrix(aspect_ratio: f32) -> glam::Mat4 {
        let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 20.0);
        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(5.0, -11.0, 3.0),
            glam::Vec3::new(1.5, 0.0, 0.0),
            glam::Vec3::Z,
        );
        projection * view
    }
}

impl framework::Example for Example {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        // Create the vertex and index buffers
        let vertex_size = mem::size_of::<shapes::Vertex>();

        // Create meshes
        let mut cube = Mesh {
            m_type: MeshType::Cube,
            vertices: Vec::new(),
            indices: Vec::new()
        };
        let mut cylinder = Mesh {
            m_type: MeshType::Cylinder,
            vertices: Vec::new(),
            indices: Vec::new()
        };
        let mut sphere = Mesh {
            m_type: MeshType::Sphere,
            vertices: Vec::new(),
            indices: Vec::new()
        };

        cube.generate_vertices();
        cylinder.generate_vertices();
        sphere.generate_vertices();

        let meshes: Vec<&Mesh> = vec![&cube, &cylinder, &sphere];

        let index_data_len = |m_type: MeshType| -> u32 {
            let length;
            match m_type {
                MeshType::Cube => length = cube.indices.len(),
                MeshType::Cylinder => length = cylinder.indices.len(),
                MeshType::Sphere => length = sphere.indices.len()
            }
            length as u32
        };

        let index_offset = |mesh_type: MeshType| -> u32 {
            let mut offset: u32 = 0;
            for m in &meshes {
                if m.m_type == mesh_type {
                    return offset;
                }
                offset += index_data_len(m.m_type);
            }
            offset
        };

        // Create batches from which the objects will be drawn on the screen
        let cube1 = Batch {
            m_type: MeshType::Cube,
            texture: TextureType::Blue,
            transform_m: vec![
                glam::Mat4::from_scale_rotation_translation(
                    glam::Vec3::ONE,
                    glam::Quat::IDENTITY,
                    glam::Vec3::new(3.0, 0.0, 0.0),
                ),
                glam::Mat4::from_scale_rotation_translation(
                    glam::Vec3::ONE,
                    glam::Quat::IDENTITY,
                    glam::Vec3::new(-3.0, 0.0, 0.0),
                )
            ]
        };
        let cylinder1 = Batch {
            m_type: MeshType::Cylinder,
            texture: TextureType::Red,
            transform_m: vec![
                glam::Mat4::from_scale_rotation_translation(
                    glam::Vec3::ONE,
                    glam::Quat::IDENTITY,
                    glam::Vec3::new(0.0, 0.0, 0.0),
                )
            ]
        };
        let sphere1 = Batch {
            m_type: MeshType::Sphere,
            texture: TextureType::Yellow,
            transform_m: vec![
                glam::Mat4::from_scale_rotation_translation(
                    glam::Vec3::ONE,
                    glam::Quat::IDENTITY,
                    glam::Vec3::new(6.0, 0.0, 0.0),
                )
            ]
        };

        let batches: Vec<&Batch> = vec![&cube1, &cylinder1, &sphere1];
        let objects: Vec<Object> = shapes::get_objects_from_batches(&batches);

        // Create one big vertex and index buffer from meshes
        let (vertex_data, index_data) = shapes::merge_index_vertex_data(&meshes);

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Textures to load
        let texture_paths: [String; 3] = [
            String::from("./examples/gpu-driven-rendering/assets/blue_texture.png"),
            String::from("./examples/gpu-driven-rendering/assets/red_texture.png"),
            String::from("./examples/gpu-driven-rendering/assets/yellow_texture.png")
        ];

        // Create pipeline layout
        let bind_group_layout0 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Camera transform (projection + view matrix): mat4x4<f32>
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                // Transformation matrices for scene objects: Array of mat4x4<f32>>
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new((64*objects.len()) as u64),
                    },
                    count: None,
                },
                // Texture array: Array of texture_2d<f32>
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: core::num::NonZeroU32::new(texture_paths.len() as u32),
                },
                // Texture sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ],
        });
        let bind_group_layout1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Texture_id: u32
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4),
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout0, &bind_group_layout1],
            push_constant_ranges: &[],
        });

        // Create textures
        let mut texture_views = Vec::new();

        for file_path in &texture_paths {
            let texture_path = Path::new(file_path);
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
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            });

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            texture_views.push(texture_view);

            queue.write_texture(
                texture.as_image_copy(),
                &texture_rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4*8),
                    rows_per_image: None,
                },
                texture_extent,
            );
        }

        // Create array of texture view references
        let texture_views_refs = texture_views.iter().collect::<Vec<_>>();

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

        // Create other resources
        let mx_total = Self::generate_matrix(config.width as f32 / config.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create storage buffer with transformation matrices for individual objects
        let matrices_bytes = shapes::merge_matrices(&objects);
        let matrices_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents: bytemuck::cast_slice(&matrices_bytes),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Create indirect data for each texture type
        let mut indirect_data: Vec<Vec<u8>> = Vec::new();
        let mut texture_id_count: Vec<u32> = Vec::new();

        for _i in 0..TEXTURE_TYPE_VARIANTS {
            indirect_data.push(Vec::new());
            texture_id_count.push(0);
        }

        let mut instance_count;
        let mut base_instance = 0;
        let mut texture_id;

        for b in &batches {
            match b.texture {
                TextureType::Blue => texture_id = 0,
                TextureType::Red => texture_id = 1,
                TextureType::Yellow => texture_id = 2,
            }
            texture_id_count[texture_id] += 1;
            instance_count = b.transform_m.len();
            indirect_data[texture_id].extend(
                wgpu::util::DrawIndexedIndirect {
                    vertex_count: index_data_len(b.m_type),
                    instance_count: instance_count as u32,
                    base_index: index_offset(b.m_type),
                    vertex_offset: 0,
                    base_instance: base_instance as u32,
                }.as_bytes()
            );
            base_instance += instance_count;
        }

        println!("{:?}", texture_id_count);

        // Create one indirect buffer for each texture type
        let mut indirect_bufs = Vec::new();

        for i in 0..TEXTURE_TYPE_VARIANTS {
            indirect_bufs.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Indirect Buffer"),
                    contents: bytemuck::cast_slice(&indirect_data[i]),
                    usage: wgpu::BufferUsages::INDIRECT,
                })
            );
        }

        // Create bind group
        let bind_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: matrices_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureViewArray(&texture_views_refs),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                }
            ],
            label: None,
        });

        // Create vector of bind groups 1
        let mut bind_groups1 = Vec::new();

        for i in 0..TEXTURE_TYPE_VARIANTS {
            bind_groups1.push(
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bind_group_layout1,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Vertex Buffer"),
                                contents: bytemuck::cast_slice(&[i]),
                                usage: wgpu::BufferUsages::UNIFORM,
                            }).as_entire_binding(),
                        }
                    ],
                    label: None,
                })
            );
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

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
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
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
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let pipeline_wire = if device.features().contains(wgt::Features::POLYGON_MODE_LINE) {
            let pipeline_wire = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_wire",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                operation: wgpu::BlendOperation::Add,
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            },
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Line,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
            Some(pipeline_wire)
        } else {
            None
        };

        // Done
        Example {
            vertex_buf,
            index_buf,
            texture_id_count,
            bind_group0,
            bind_groups1,
            uniform_buf,
            indirect_bufs,
            pipeline,
            pipeline_wire,
        }
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let mx_total = Self::generate_matrix(config.width as f32 / config.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(mx_ref));
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
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
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group0, &[]);
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.pop_debug_group();
            rpass.push_debug_group("Draw!");

            for i in 0..TEXTURE_TYPE_VARIANTS {
                rpass.set_bind_group(1, &self.bind_groups1[i], &[]);
                if self.texture_id_count[i] > 0 {
                    rpass.multi_draw_indexed_indirect(&self.indirect_bufs[i], 0, self.texture_id_count[i]);
                }
            }

            // Pipeline wire
            if let Some(ref pipe) = self.pipeline_wire {
                rpass.set_pipeline(pipe);
                for i in 0..TEXTURE_TYPE_VARIANTS {
                    if self.texture_id_count[i] > 0 {
                        rpass.multi_draw_indexed_indirect(&self.indirect_bufs[i], 0, self.texture_id_count[i]);
                    }
                }
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}

fn main() {
    framework::run::<Example>("GPU driven rendering");
}
