const SAMPLE_COUNT: u32 = 8;

struct _Surface {
    surface: wgpu::Surface,
    multi_sampled_texture: Option<wgpu::TextureView>,
    config: wgpu::SurfaceConfiguration,
}

impl _Surface {
    fn _new(
        raw_surface: &wgpu::Surface,
        adapter: &Adapter,
        size: PhysicalSize<u32>,
        device: &_Device,
    ) -> Self {
        let capabilities = raw_surface.get_capabilities(&adapter);
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
        raw_surface.configure(&device.device, &config);
        let multi_sampled_texture =
            _Surface::create_multisampled_framebuffer(&device.device, &config, SAMPLE_COUNT);

        todo!()
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
}

struct _Device {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl _Device {
    async fn _new(adapter: &Adapter) -> Self {
        let limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };

        let descriptor = wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: limits,
            label: None,
        };

        let (device, queue) = adapter.request_device(&descriptor, None).await.unwrap();
        Self { device, queue }
    }
}

struct _Engine {
    window: Window,
    surface: _Surface,
    device: _Device,
    engine_objects: Vec<Box<dyn EngineObject>>,
    time: Time,
    uniform_buffer: UniformBuffer,
    size: winit::dpi::PhysicalSize<u32>,
}

impl _Engine {
    async fn _new() -> (Self, EventLoop<()>) {
        let (window, event_loop) = create_window();
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let raw_surface = unsafe { instance.create_surface(&window).unwrap() };
        let adapter = Self::_create_adapter(&raw_surface, &instance).await;
        let device = _Device::_new(&adapter).await;
        let engine_objects = vec![];
        let surface = _Surface::_new(&raw_surface, &adapter, size, &device);
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
            },
            event_loop,
        )
    }

    async fn _create_adapter(surface: &wgpu::Surface, instance: &Instance) -> Adapter {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap()
    }
}

mod time;
mod uniforms;

use std::{iter, marker::PhantomData};
use time::Time;
#[allow(unused_imports)]
use tracing::{error, info, warn};
use uniforms::UniformBuffer;
use wgpu::{Adapter, BindGroupLayout, Device, Instance, TextureFormat};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder, WindowId},
};

#[cfg(target_arch = "wasm32")]
const WIDTH: u32 = 2048;
#[cfg(target_arch = "wasm32")]
const HEIGHT: u32 = 1200;

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

pub struct Surface {
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    multi_sampled_texture: wgpu::TextureView,
}

impl Surface {
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
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        info!("Device: {:?}", device);

        let capabilities = surface.get_capabilities(&adapter);

        info!("Capabilities: {:?}", capabilities);

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

        surface.configure(&device, &config);

        let multi_sampled_texture = Self::create_multisampled_framebuffer(&device, &config, 8);

        Self {
            size,
            surface,
            device,
            config,
            queue,
            multi_sampled_texture,
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

pub struct Application {
    pub engine: Engine,
    event_loop: EventLoop<()>,
}

impl Application {
    pub fn surface(&self) -> &Surface {
        &self.engine.surface
    }

    pub async fn new() -> Application {
        let (window, event_loop) = create_window();
        let engine = Engine::new(window).await;

        Self { engine, event_loop }
    }

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

    pub fn add_engine_object(&mut self, object: Box<dyn EngineObject>) {
        self.engine.add_engine_object(object);
    }
}

pub struct Engine {
    surface: Surface,
    window: Window,
    time: Time,
    uniform_buffer: UniformBuffer,
    engine_objects: Vec<Box<dyn EngineObject>>,
}

impl Engine {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.surface.resize(new_size);
        let size = self.window.inner_size();
        self.uniform_buffer.update_width(size.width as f32);
        self.uniform_buffer.update_height(size.height as f32);
    }

    pub async fn new(window: Window) -> Engine {
        let surface = Surface::new(&window).await;
        let engine_objects = vec![];
        let uniform_buffer = UniformBuffer::new(&surface.device, &window);
        let time = Time::new();

        Self {
            surface,
            window,
            time,
            uniform_buffer,
            engine_objects,
        }
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

    fn handle_redraw_requested(&mut self, window_id: WindowId, control_flow: &mut ControlFlow) {
        let delta_time = self.time.tick();
        if window_id == self.window.id() {
            self.update(delta_time);
            match self.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => self.resize(self.surface.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.uniform_buffer.update_run_time(self.time.run_time());
        self.uniform_buffer.update_delta_time(delta_time);

        self.surface.queue.write_buffer(
            &self.uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[self.uniform_buffer.uniforms]),
        );
        self.engine_objects
            .iter_mut()
            .for_each(|object| object.update());
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.surface
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &self.surface.multi_sampled_texture,
                resolve_target: Some(&view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: false,
                },
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
            });

            render_pass.set_bind_group(0, &self.uniform_buffer.bind_group, &[]);

            for object in &self.engine_objects {
                if let Some(render_data) = object.render() {
                    render_pass.set_pipeline(render_data.render_pipeline);
                    render_pass.set_vertex_buffer(0, render_data.vertex_buffer.slice(..));
                    match render_data.index_buffer {
                        None => render_pass.draw(0..render_data.num_vertices, 0..0),
                        Some(index_buffer) => {
                            render_pass.set_index_buffer(
                                index_buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );
                            render_pass.draw_indexed(0..render_data.num_indices, 0, 0..1)
                        }
                    }
                }
            }
        }

        self.surface.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn add_engine_object(&mut self, object: Box<dyn EngineObject>) {
        self.engine_objects.push(object);
    }

    pub fn uniform_bind_group(&self) -> &BindGroupLayout {
        &self.uniform_buffer.bind_group_layout
    }

    pub fn device(&self) -> &Device {
        &self.surface.device
    }

    pub fn surface_format(&self) -> &TextureFormat {
        &self.surface.config.format
    }
}

fn create_window() -> (Window, EventLoop<()>) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
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
