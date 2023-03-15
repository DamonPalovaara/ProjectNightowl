use wgpu::util::DeviceExt;

use crate::engine::{Engine, EngineObject, RenderData};
use crate::types::Vertex3;

#[rustfmt::skip]
const SCREEN: &[Vertex3] = &[
    Vertex3::new([-1.0, -1.0, 0.0]),
    Vertex3::new([-1.0,  1.0, 0.0]),
    Vertex3::new([ 1.0,  1.0, 0.0]),
    Vertex3::new([ 1.0, -1.0, 0.0]),
];

const SCREEN_INDICES: &[u16] = &[0, 3, 2, 2, 1, 0];

pub struct ComplexGrapher {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl ComplexGrapher {
    pub fn _new(engine: &Engine) -> Self {
        let device = engine.device();
        let shader = device.create_shader_module(wgpu::include_wgsl!("./complex.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Complex Layout"),
                bind_group_layouts: &[engine.uniform_bind_group()],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Complex Graph Layout"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex3::desc()],
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
                count: engine.sample_count(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: *engine.surface_format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Clear Screen"),
            contents: bytemuck::cast_slice(SCREEN),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Clear Screen"),
            contents: bytemuck::cast_slice(SCREEN_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }
}

impl EngineObject for ComplexGrapher {
    fn render(&self) -> Option<RenderData> {
        Some(RenderData {
            render_pipeline: &self.render_pipeline,
            vertex_buffer: &self.vertex_buffer,
            index_buffer: Some(&self.index_buffer),
            num_vertices: 4,
            num_indices: 6,
        })
    }
}
