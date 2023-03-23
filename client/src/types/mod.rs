use bytemuck::{Pod, Zeroable};
use paste::paste;
use std::mem::size_of;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat::*, VertexStepMode};

#[macro_use]
mod macros;

vertex_struct!(Vertex2, pos: [f32; 2]);
vertex_struct!(Vertex3, pos: [f32; 3]);

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_vertex_attributes() {
        vertex_struct!(
            TestVertex,
            position: [f32; 3],
            normal: [f32; 3],
            color: [u8; 4],
            uv: [f32; 2]
        );

        let expected_attributes = [
            VertexAttribute {
                format: Float32x3,
                offset: 0,
                shader_location: 0,
            },
            VertexAttribute {
                format: Float32x3,
                offset: 12,
                shader_location: 1,
            },
            VertexAttribute {
                format: Uint8x4,
                offset: 24,
                shader_location: 2,
            },
            VertexAttribute {
                format: Float32x2,
                offset: 28,
                shader_location: 3,
            },
        ];

        assert_eq!(size_of::<TestVertex>(), 36);
        assert_eq!(TestVertex::ATTRIBUTES.len(), 4);
        assert_eq!(TestVertex::ATTRIBUTES, expected_attributes);
    }
}
