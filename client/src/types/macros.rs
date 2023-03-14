// Helper macro to match Rust type with webgpu SIMD type
#[rustfmt::skip]
macro_rules! match_type {
    (f32, 1) => { VertexFormat::Float32   };
    (f32, 2) => { VertexFormat::Float32x2 };
    (f32, 3) => { VertexFormat::Float32x3 };
    (f32, 4) => { VertexFormat::Float32x4 };
    (f64, 1) => { VertexFormat::Float64   };
    (f64, 2) => { VertexFormat::Float64x2 };
    (f64, 3) => { VertexFormat::Float64x3 };
    (f64, 4) => { VertexFormat::Float64x4 };
    (u8,  2) => { VertexFormat::Uint8x2   };
    (u8,  4) => { VertexFormat::Uint8x4   };
    (i8,  2) => { VertexFormat::Sint8x2   };
    (i8,  4) => { VertexFormat::Sint8x4   };
    (u16, 2) => { VertexFormat::Uint16x2  };
    (u16, 4) => { VertexFormat::Uint16x4  };
    (i16, 2) => { VertexFormat::Sint16x2  };
    (i16, 4) => { VertexFormat::Sint16x4  };
    (u32, 1) => { VertexFormat::Uint32    };
    (u32, 2) => { VertexFormat::Uint32x2  };
    (u32, 3) => { VertexFormat::Uint32x3  };
    (u32, 4) => { VertexFormat::Uint32x4  };
    (u64, 1) => { VertexFormat::Uint64    };
    (u64, 2) => { VertexFormat::Uint64x2  };
    (u64, 3) => { VertexFormat::Uint64x3  };
    (u64, 4) => { VertexFormat::Uint64x4  };
}

// Generates a type that can be sent to the GPU
macro_rules! vertex_struct {
    ($name:ident, $($field:ident: [$type:tt; $size:tt]),*) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, Pod, Zeroable, Default)]
        pub struct $name {
            $(pub $field: [$type; $size]),*
        }

        #[allow(dead_code)]
        impl $name {
            const ATTRIBUTES: [ VertexAttribute; count!($($size),*) ] = {
                // Underscore to ignore clippy unused_assignment lints
                let mut _shader_location = 0;
                let mut _offset: u64 = 0;
                [$(
                    {
                        let offset = _offset;
                        let shader_location = _shader_location;
                        let format = match_type!($type, $size);
                        _offset += std::mem::size_of::<[$type; $size]>() as u64;
                        _shader_location += 1;
                        VertexAttribute { format, offset, shader_location }
                    }
                ,)*]
            };

            pub fn desc<'a>() -> VertexBufferLayout<'a> {
                VertexBufferLayout {
                    array_stride: size_of::<Self>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &Self::ATTRIBUTES,
                }
            }

            pub const fn new($($field: [$type; $size]),*) -> Self {
                Self { $($field),* }
            }
        }
    };
}

macro_rules! count {
    () => { 0 };
    ($first:literal $(, $rest:literal)*) => {
        1 + count!($($rest),*)
    };
}
