#[rustfmt::skip]
macro_rules! match_type {
    (f32, 1) => { Float32 }; 
    (f64, 1) => { Float64 };
    (u32, 1) => { Uint32  };
    (i32, 1) => { Sint32  };
    (f32, $n:tt) => { paste!{ [<Float32x $n>] } };
    (f64, $n:tt) => { paste!{ [<Float64x $n>] } };
    (u8,  $n:tt) => { paste!{ [<Uint8x   $n>] } };
    (i8,  $n:tt) => { paste!{ [<Sint8x   $n>] } };
    (u16, $n:tt) => { paste!{ [<Uint16x  $n>] } };
    (i16, $n:tt) => { paste!{ [<Sint16x  $n>] } };
    (u32, $n:tt) => { paste!{ [<Uint32x  $n>] } };
    (i32, $n:tt) => { paste!{ [<Sint32x  $n>] } };
}

// Generates a type that can be sent to the GPU
macro_rules! vertex_struct {
    ($name:ident, $($field:ident: [$type:tt; $size:tt]),* $(,)?) => {
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
