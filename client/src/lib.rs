use tracing::{error, info, warn};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder, WindowId},
};

use wgpu::util::DeviceExt;

trait Render {
    fn render(&self, surface: &Surface) -> Result<(), wgpu::SurfaceError>;
}

trait Update {
    fn update(&mut self);
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

struct Surface {
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl Surface {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(window).unwrap() };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let capabilities = surface.get_capabilities(&adapter);

        let surface_format = capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Self {
            size,
            surface,
            device,
            config,
            queue,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

struct Pipeline {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Pipeline {
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",     // 1.
                buffers: &[Vertex::desc()], // 2.
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }
}

impl Render for Pipeline {
    fn render(&self, surface: &Surface) -> Result<(), wgpu::SurfaceError> {
        let output = surface.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = surface
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        surface.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

struct State {
    pipeline: Pipeline,
}

impl State {
    async fn new(window: &Window, surface: &Surface) -> Self {
        let pipeline = Pipeline::new(&surface.device, &surface.config);

        Self { pipeline }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}
}

impl Render for State {
    fn render(&self, surface: &Surface) -> Result<(), wgpu::SurfaceError> {
        self.pipeline.render(surface)?;
        Ok(())
    }
}

fn create_window() -> (Window, EventLoop<()>) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(640, 480));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to div.");
    }

    (window, event_loop)
}

struct Application {
    engine: Engine,
    event_loop: EventLoop<()>,
}

impl Application {
    pub async fn new() -> Self {
        // Initialize the logger
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                console_error_panic_hook::set_once();
                tracing_wasm::set_as_global_default();
            }
            else {
                tracing_subscriber::fmt::init();
            }
        }

        let (window, event_loop) = create_window();
        let engine = Engine::new(window).await;

        Self { engine, event_loop }
    }
}

impl Application {
    pub fn run(mut self) {
        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::WindowEvent { window_id, event } => {
                    self.engine
                        .handle_window_event(window_id, &event, control_flow);
                }
                Event::RedrawRequested(window_id) => {
                    self.engine.handle_redraw_requested(window_id, control_flow);
                }
                Event::MainEventsCleared => {
                    self.engine.window.request_redraw();
                }
                _ => {}
            });
    }

    fn add_updatable(&mut self, object: Box<dyn Update>) {
        self.engine.add_updatable(object);
    }

    fn add_renderable(&mut self, object: Box<dyn Render>) {
        self.engine.add_renderable(object);
    }
}

struct Engine {
    surface: Surface,
    state: State,
    window: Window,
    updatable: Vec<Box<dyn Update>>,
    renderable: Vec<Box<dyn Render>>,
}

impl Engine {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.surface.resize(new_size);
    }

    pub async fn new(window: Window) -> Self {
        let surface = Surface::new(&window).await;
        let state = State::new(&window, &surface).await;
        let updatable = vec![];
        let renderable = vec![];

        Self {
            surface,
            state,
            window,
            updatable,
            renderable,
        }
    }

    fn handle_window_event(
        &mut self,
        window_id: WindowId,
        event: &WindowEvent,
        control_flow: &mut ControlFlow,
    ) {
        if window_id == self.window.id() {
            if !self.state.input(event) {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        self.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.resize(**new_inner_size);
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
        }
    }

    fn handle_redraw_requested(&mut self, window_id: WindowId, control_flow: &mut ControlFlow) {
        if window_id == self.window.id() {
            // self.state.update();
            // match self.state.render() {
            self.update();
            match self.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => self.resize(self.surface.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderable
            .iter()
            .try_for_each(|object| object.render(&self.surface));
        Ok(())
    }

    fn add_updatable(&mut self, object: Box<dyn Update>) {
        self.updatable.push(object);
    }

    fn add_renderable(&mut self, object: Box<dyn Render>) {
        self.renderable.push(object);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    let mut app = Application::new().await;
    let state = State::new(&app.engine.window, &app.engine.surface).await;
    app.add_renderable(Box::new(state));
    app.run();
}
