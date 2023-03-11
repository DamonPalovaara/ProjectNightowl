use wgpu::util::DeviceExt;

use super::engine::{Engine, Render, RenderData, Vertex};

const CLEAR_SCREEN: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
        color: [0.9, 0.1, 0.1],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        color: [0.1, 0.9, 0.1],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        color: [0.1, 0.1, 0.9],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        color: [0.8, 0.8, 0.8],
    },
];

const CLEAR_INDICES: &[u16] = &[0, 3, 2, 2, 1, 0];

pub struct ComplexGrapher {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl ComplexGrapher {
    pub fn new(engine: &Engine) -> Self {
        let device = &engine.surface.device;
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/complex.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Complex Layout"),
                bind_group_layouts: &[&engine.uniform_buffer.bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Complex Graph Layout"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 8,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: engine.surface.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Clear Screen"),
            contents: bytemuck::cast_slice(CLEAR_SCREEN),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Clear Screen"),
            contents: bytemuck::cast_slice(CLEAR_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }
}

impl Render for ComplexGrapher {
    fn render(&self) -> RenderData {
        RenderData {
            render_pipeline: &self.render_pipeline,
            vertex_buffer: &self.vertex_buffer,
            index_buffer: Some(&self.index_buffer),
            num_vertices: 4,
            num_indices: 6,
        }
    }
}
