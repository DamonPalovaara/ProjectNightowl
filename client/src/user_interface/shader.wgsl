struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

fn construct_matrix() -> mat3x3<f32> {
    // TODO: Get from engines uniform buffer
    let screen_size = vec2(1800.0, 1000.0);
    // TODO: Pass in via instance buffer
    let rect_size = vec2(400.0, 200.0);
    let pos = vec2(100.0, 100.0);

    let scale = 2.0 * (rect_size / screen_size);
    let translation = vec2(-1.0) + (2.0 * pos / screen_size);

    return mat3x3(
        vec3(scale.x, 0.0, 0.0), 
        vec3(0.0, scale.y, 0.0), 
        vec3(translation, 1.0),
    );
}


@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let translated = construct_matrix() * vec3(vertex.position, 1.0);
    out.position = vec4(translated.xy, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 0.75);
}