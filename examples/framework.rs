use winit::event_loop::{ EventLoop, ControlFlow };
use winit::event;
use winit::event::WindowEvent;
use std::time::Instant;

pub trait Example: 'static + Sized {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self;
    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );
    fn update(&mut self, event: WindowEvent);
    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        spawner: &Spawner,
    );
}

struct Setup {
    window: winit::window::Window,
    event_loop: EventLoop<()>,
    instance: wgpu::Instance,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

pub struct Spawner<'a> {
    executor: async_executor::LocalExecutor<'a>,
}

impl<'a> Spawner<'a> {
    fn new() -> Self {
        Self {
            executor: async_executor::LocalExecutor::new(),
        }
    }

    fn run_until_stalled(&self) {
        while self.executor.try_tick() {}
    }
}

async fn setup<E: Example>(title: &str) -> Setup {
    // Create window builder to build our window
    let event_loop = EventLoop::new();
    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title(title);

    // Building the window
    let window = builder.build(&event_loop).unwrap();

    // Describes which backends we want to use (Vulkan, DirectX11, ...)
    let backend = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
    let instance = wgpu::Instance::new(backend); // Create WGPU instance (context for all other wgpu objects)

    // Physical size of window's client area (content of the window, excluding the title bar and borders)
    let size = window.inner_size();
    // Surface represents a surface (window) onto which rendered images may be presented
    let surface = unsafe {
        let window_surface = instance.create_surface(&window);
        window_surface
    };

    // Handle to our graphical or compute device
    let adapter = wgpu::util::initialize_adapter_from_env_or_default(
        &instance,
        backend,
        Some(&surface),
    ).await.expect("No suitable GPU adapters on the system!");
    
    // Device represents a connection to our graphics or compute device
    // It is responsible of most rendering and compute resources
    // Queue is a representation of a queue of work that is submitted to the GPU for execution
    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: adapter.features(), // Set features to adapter features
            limits: adapter.limits() // Sets the limits to the limits of our actual graphics adapter
        },
        None
    ).await.expect("Unable to find suitable GPU adapter!");

    Setup {
        window,
        event_loop,
        instance,
        size,
        surface,
        adapter,
        device,
        queue
    }
}

fn start<E: Example>(
    Setup {
        window,
        event_loop,
        instance,
        size,
        surface,
        adapter,
        device,
        queue,
    }: Setup,
) {
    let spawner = Spawner::new();

    // Configuration of the surface
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        format: surface.get_supported_formats(&adapter)[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
    };
    surface.configure(&device, &config);

    let mut last_frame_inst = Instant::now();
    let (mut frame_count, mut accum_time) = (0.0, 0.0);

    let mut example = E::init(&config, &adapter, &device, &queue);

    event_loop.run(move |event, _, control_flow| {
        let _ = (&instance, &adapter); // force ownership by the closure
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            event::Event::RedrawEventsCleared => {
                spawner.run_until_stalled();
                window.request_redraw();
            }
            event::Event::WindowEvent {
                event:
                    WindowEvent::Resized(size)
                    | WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut size,
                        ..
                    },
                ..
            } => {
                config.width = size.width.max(1);
                config.height = size.height.max(1);
                example.resize(&config, &device, &queue);
                surface.configure(&device, &config);
            }
            event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Escape),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::R),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    println!("{:#?}", instance.generate_report());
                }
                _ => {
                    example.update(event);
                }
            },
            event::Event::RedrawRequested(_) => {
                {
                    accum_time += last_frame_inst.elapsed().as_secs_f32();
                    last_frame_inst = Instant::now();
                    frame_count += 1.0;
                    if frame_count == 100.0 {
                        println!(
                            "Average fps: {}",
                            frame_count / accum_time
                        );
                        accum_time = 0.0;
                        frame_count = 0.0;
                    }
                }

                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        surface.configure(&device, &config);
                        surface.get_current_texture().expect("Failed to acquire next surface texture!")
                    }
                };

                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                example.render(&view, &device, &queue, &config, &spawner);
                frame.present();
            }
            _ => {}
        }
    });
}

pub fn run<E: Example>(title: &str) {
    let setup = pollster::block_on(setup::<E>(title));
    start::<E>(setup);
}

#[allow(dead_code)]
fn main() {}