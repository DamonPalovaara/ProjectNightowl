#[cfg(target_arch = "wasm32")]
const WIDTH: u32 = 1800;
#[cfg(target_arch = "wasm32")]
const HEIGHT: u32 = 1000;

mod time;
mod uniforms;

use std::iter;
use time::Time;
#[allow(unused_imports)]
use tracing::{error, info, warn};
use uniforms::UniformBuffer;
use wgpu::{Adapter, BindGroupLayout, Instance, TextureFormat};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder, WindowId},
};

pub struct EngineConfig {
    pub msaa: Option<u32>,
}

struct Surface {
    surface: wgpu::Surface,
    multi_sampled_texture: Option<wgpu::TextureView>,
    config: wgpu::SurfaceConfiguration,
}

impl Surface {
    fn new(
        surface: wgpu::Surface,
        adapter: &Adapter,
        size: PhysicalSize<u32>,
        device: &Device,
        engine_config: &EngineConfig,
    ) -> Self {
        let capabilities = surface.get_capabilities(adapter);
        let surface_format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.describe().srgb)
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
        surface.configure(&device.device, &config);

        let multi_sampled_texture = match engine_config.msaa {
            Some(sample_count) => Some(create_multisampled_framebuffer(
                &device.device,
                &config,
                sample_count,
            )),
            None => None,
        };

        Self {
            surface,
            multi_sampled_texture,
            config,
        }
    }

    fn resize(&mut self, new_size: &PhysicalSize<u32>, device: &Device) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&device.device, &self.config);
    }

    fn get_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }
}

struct Device {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Device {
    async fn new(adapter: &Adapter) -> Self {
        let limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };

        let descriptor = wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits,
            label: None,
        };

        let (device, queue) = adapter.request_device(&descriptor, None).await.unwrap();
        Self { device, queue }
    }

    fn create_encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main render encoder"),
            })
    }

    fn submit<I>(&self, command_buffers: I)
    where
        I: IntoIterator<Item = wgpu::CommandBuffer>,
    {
        self.queue.submit(command_buffers);
    }
}

pub struct Engine {
    window: Window,
    surface: Surface,
    device: Device,
    engine_objects: Vec<Box<dyn EngineObject>>,
    time: Time,
    uniform_buffer: UniformBuffer,
    size: winit::dpi::PhysicalSize<u32>,
    config: EngineConfig,
}

impl Engine {
    pub async fn new(config: EngineConfig) -> (Self, EventLoop<()>) {
        let (window, event_loop) = create_window();
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(&window).unwrap() };
        let adapter = create_adapter(&surface, &instance).await;
        let device = Device::new(&adapter).await;
        let engine_objects = vec![];
        let surface = Surface::new(surface, &adapter, size, &device, &config);
        let time = Time::new();
        let uniform_buffer = UniformBuffer::new(&device.device, &window);

        (
            Self {
                window,
                surface,
                device,
                engine_objects,
                time,
                uniform_buffer,
                size,
                config,
            },
            event_loop,
        )
    }

    fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = *new_size;
            self.surface.resize(new_size, &self.device);
            self.uniform_buffer.update_width(self.size.width as f32);
            self.uniform_buffer.update_height(self.size.height as f32);
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { window_id, event } => {
                self.handle_window_event(window_id, &event, control_flow);
            }
            Event::RedrawRequested(window_id) => {
                self.handle_redraw_requested(window_id, control_flow);
            }
            Event::MainEventsCleared => {
                self.window.request_redraw();
            }
            _ => (),
        });
    }

    pub fn add_engine_object(&mut self, object: Box<dyn EngineObject>) {
        self.engine_objects.push(object);
    }

    fn handle_window_event(
        &mut self,
        window_id: WindowId,
        event: &WindowEvent,
        control_flow: &mut ControlFlow,
    ) {
        if window_id == self.window.id() {
            match event {
                WindowEvent::Resized(physical_size) => {
                    self.resize(physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.resize(new_inner_size)
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
                _ => (),
            }
        }
    }

    fn handle_redraw_requested(&mut self, window_id: WindowId, control_flow: &mut ControlFlow) {
        if window_id == self.window.id() {
            let delta_time = self.time.tick();
            self.update(delta_time);
            match self.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => self.resize(&(self.size.clone())),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.uniform_buffer.update_run_time(self.time.run_time());
        self.uniform_buffer.update_delta_time(delta_time);
        self.uniform_buffer.write(&mut self.device.queue);
        self.engine_objects
            .iter_mut()
            .for_each(|object| object.update());
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_encoder();
        {
            let color_attachment = match &self.surface.multi_sampled_texture {
                Some(multi_sampled_texture) => wgpu::RenderPassColorAttachment {
                    view: multi_sampled_texture,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: false,
                    },
                },
                None => wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: false,
                    },
                },
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
            });

            render_pass.set_bind_group(0, &self.uniform_buffer.bind_group, &[]);

            self.engine_objects.iter().for_each(|object| {
                if let Some(render_data) = object.render() {
                    render_pass.set_pipeline(render_data.render_pipeline);
                    render_pass.set_vertex_buffer(0, render_data.vertex_buffer.slice(..));
                    match render_data.index_buffer {
                        None => {
                            render_pass.draw(0..render_data.num_vertices, 0..0);
                        }
                        Some(index_buffer) => {
                            render_pass.set_index_buffer(
                                index_buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            render_pass.draw_indexed(0..render_data.num_indices, 0, 0..1);
                        }
                    }
                }
            })
        }
        self.device.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn sample_count(&self) -> u32 {
        if let Some(sample_count) = self.config.msaa {
            sample_count
        } else {
            1
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device.device
    }

    pub fn surface_format(&self) -> &TextureFormat {
        &self.surface.config.format
    }

    pub fn uniform_bind_group(&self) -> &BindGroupLayout {
        &self.uniform_buffer.bind_group_layout
    }
}

/// Contains all the methods the engine will call on EngineObject
/// Notice that none of the methods are required, that allows for
/// flexibility with that don't need to render and such.
pub trait EngineObject {
    fn start(&mut self, _engine: &Engine) {}
    fn update(&mut self) {}
    fn render(&self) -> Option<RenderData> {
        None
    }
}

/// Contains all the data that the engine requires to draw an object
pub struct RenderData<'a> {
    pub render_pipeline: &'a wgpu::RenderPipeline,
    pub vertex_buffer: &'a wgpu::Buffer,
    pub index_buffer: Option<&'a wgpu::Buffer>,
    pub num_vertices: u32,
    pub num_indices: u32,
}

fn create_window() -> (Window, EventLoop<()>) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        window.set_inner_size(PhysicalSize::new(WIDTH, HEIGHT));

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

fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    sample_count: u32,
) -> wgpu::TextureView {
    let multisampled_texture_extent = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        size: multisampled_texture_extent,
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[],
    };
    device
        .create_texture(multisampled_frame_descriptor)
        .create_view(&wgpu::TextureViewDescriptor::default())
}

async fn create_adapter(surface: &wgpu::Surface, instance: &Instance) -> Adapter {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap()
}
