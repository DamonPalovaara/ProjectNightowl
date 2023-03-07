
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

// f(x, n) = (x mod n) / n
fn norm_mod_n(x: f32, n: f32) -> f32 {
    let y = x - floor(x / n) * n;
    if (y < 0.0) {
        return (y + n) / n;
    }
    else {
        return y / n;
    }
}

// f(r, t, n) = (r^n)*e^(i*n*t)
fn z_n(r: f32, t: f32, n: f32) -> vec2<f32> {
    return vec2(pow(r, n) * cos(n * t), pow(r, n) * sin(n * t));
}

// f(x, y) = e^(x + iy)
fn e_z(x: f32, y: f32) -> vec2<f32> {
    return vec2(exp(x) * cos(y), exp(x) * sin(y));
}

// Converts rectangular to polar
fn xy_to_rt(x: f32, y: f32) -> vec2<f32> {
    return vec2(pow(x * x + y * y, 0.5), atan2(y, x));
}

fn foo(x: f32, y: f32) -> vec2<f32> {
    return vec2(y * x * x, x / 2.0);
}

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let scale = 10.0;
    let pos = in.clip_position.xy - vec2(900.0, 400.0);

    let z = (pos / 900.0) * scale;
    // let rt = xy_to_rt(z.x, z.y);
    let uv = e_z(z.x, z.y);
    // let uv = z_n(rt.x, rt.y, 2.0);

    let n = 1.0;
    let u = norm_mod_n(uv[0], n);
    let v = norm_mod_n(uv[1], n);


    // let u = uv.x;
    // let v = uv.y;

    let u_pow = vec3(2.0, 0.0, 0.0);
    let v_pow = vec3(0.0, 2.0, 0.0);

    return vec4(
        pow(u, u_pow[0]) * pow(v, v_pow[0]), 
        pow(u, u_pow[1]) * pow(v, v_pow[1]), 
        pow(u, u_pow[2]) * pow(v, v_pow[2]), 
        1.0
    );
}