use wgpu::{util::DeviceExt, Device};
use winit::window::Window;

/// Engine level global Uniforms that can be used in any shader
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Uniforms {
    delta_time: f32,
    run_time: f32,
    width: f32,
    height: f32,
}

#[derive(Debug)]
pub struct UniformBuffer {
    pub uniforms: Uniforms,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl UniformBuffer {
    pub fn new(device: &Device, window: &Window) -> Self {
        let mut uniforms = Uniforms::default();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("uniform_buffer_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let size = window.inner_size();
        uniforms.width = size.width as f32;
        uniforms.height = size.height as f32;

        Self {
            uniforms,
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn update_run_time(&mut self, run_time: f32) {
        self.uniforms.run_time = run_time;
    }

    pub fn update_delta_time(&mut self, delta_time: f32) {
        self.uniforms.delta_time = delta_time;
    }

    pub fn update_width(&mut self, width: f32) {
        self.uniforms.width = width;
    }

    pub fn update_height(&mut self, height: f32) {
        self.uniforms.height = height;
    }

    pub fn write(&self, queue: &mut wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
    }
}
