use wgpu::util::DeviceExt;

use crate::engine::{Engine, EngineObject, RenderData};
use crate::types::Vertex3;

#[rustfmt::skip]
const TEMP: &[Vertex3] = &[
    Vertex3::new([-0.25, -0.25, 0.0]),
    Vertex3::new([ 0.25, -0.25, 0.0]),
    Vertex3::new([-0.25,  0.25, 0.0]),
    Vertex3::new([ 0.25,  0.25, 0.0]),
];

const TEMP_INDICES: &[u16] = &[0, 1, 2, 3];

pub struct UserInterface {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl UserInterface {
    pub fn new(engine: &Engine) -> Self {
        let device = engine.device();
        let shader = device.create_shader_module(wgpu::include_wgsl!("./shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("UI Layout"),
                bind_group_layouts: &[engine.uniform_bind_group()],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Descriptor"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex3::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
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
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),

                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Vertex Buffer"),
            contents: bytemuck::cast_slice(TEMP),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Index Buffer"),
            contents: bytemuck::cast_slice(TEMP_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }
}

impl EngineObject for UserInterface {
    fn render(&self) -> Option<RenderData> {
        Some(RenderData {
            render_pipeline: &self.render_pipeline,
            vertex_buffer: &self.vertex_buffer,
            index_buffer: Some(&self.index_buffer),
            num_vertices: 4,
            num_indices: TEMP_INDICES.len() as u32,
        })
    }
}
