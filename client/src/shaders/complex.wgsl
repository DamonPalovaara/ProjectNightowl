
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>
};


@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}


@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let size = vec2(1280.0, 720.0);
    let z = in.clip_position.xy / size;

    let u = (z.x * z.x) - (z.y * z.y);
    let v = 2.0 * z.x * z.y;

    return vec4(z.x, (z.x + z.y) / 2.0, z.y, 0.0);
}