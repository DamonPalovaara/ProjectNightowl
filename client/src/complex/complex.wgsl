
struct VertexInput {
    @location(0) pos: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct Uniforms {
    delta_time: f32,
    run_time: f32,
    width: f32,
    height: f32
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.pos, 1.0);
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

fn mod_n(x: f32, n: f32) -> f32 {
    let y = x - floor(x / n) * n;
    if (y < 0.0) {
        return (y + n);
    }
    else {
        return y;
    }
}

fn rgb_to_hsv(input: vec3<f32>) -> vec3<f32> {
	// Extract the red, green, and blue components from the input color
	let r = input.r;
	let g = input.g;
	let b = input.b;

	// Find the minimum and maximum values among the three components
	let cmin = min(min(r, g), b);
	let cmax = max(max(r, g), b);

	// Calculate the difference between the minimum and maximum values
	let delta = cmax - cmin;

	// Calculate the hue value
	var hue: f32 = 0.0;
	if delta != 0.0 {
		if cmax == r {
			hue = (g - b) / delta;
		} else if cmax == g {
			hue = 2.0 + (b - r) / delta;
		} else {
			hue = 4.0 + (r - g) / delta;
		}
		hue = hue / 6.0;
		if hue < 0.0 {
			hue = hue + 1.0;
		}
	}

	// Calculate the saturation value
	var saturation: f32 = 0.0;
	if cmax != 0.0 {
		saturation = delta / cmax;
	}

	// Calculate the value (brightness) value
	let value = cmax;

	// Return the HSV color as a vec3
	return vec3<f32>(hue, saturation, value);
}

fn hsv_to_rgb(input: vec3<f32>) -> vec3<f32> {
	// Extract the hue, saturation, and value components from the input color
	let h = input.r;
	let s = input.g;
	let v = input.b;

	// Calculate the chroma value
	let c = v * s;

	// Calculate the hue sector
	let hs = h * 6.0;

	// Calculate the second largest component
	let x = c * (1.0 - abs(mod_n(hs, 2.0) - 1.0));

	// Calculate the base RGB values
	var r: f32 = 0.0;
	var g: f32 = 0.0;
	var b: f32 = 0.0;
	if hs < 1.0 {
		r = c;
		g = x;
	} else if hs < 2.0 {
		r = x;
		g = c;
	} else if hs < 3.0 {
		g = c;
		b = x;
	} else if hs < 4.0 {
		g = x;
		b = c;
	} else if hs < 5.0 {
		r = x;
		b = c;
	} else {
		r = c;
		b = x;
	}

	// Calculate the RGB values with the correct offsets
	let m = v - c;
	r = r + m;
	g = g + m;
	b = b + m;

	// Return the RGB color as a vec3
	return vec3<f32>(r, g, b);
}


// f(r, t, n) = (r^n)*e^(i*n*t)
fn z_n(r: f32, t: f32, n: f32) -> vec2<f32> {
    return vec2(pow(r, n) * cos(n * t), pow(r, n) * sin(n * t));
}

fn ln_z(r: f32, t: f32) -> vec2<f32> {
	return vec2(log(r), t);
}

// f(x, y) = e^(x + iy)
fn e_z(x: f32, y: f32) -> vec2<f32> {
    return vec2(exp(x) * cos(y), exp(x) * sin(y));
}

// Converts rectangular to polar
fn xy_to_rt(x: f32, y: f32) -> vec2<f32> {
    return vec2(pow(x * x + y * y, 0.5), atan2(y, x));
}

fn cos_z(x: f32, y: f32) -> vec2<f32> {
	let u = cos(x) * cosh(y);
	let v = -1.0 * sin(x) * sinh(y);
	return vec2(u, v);
}

fn sin_z(x: f32, y: f32) -> vec2<f32> {
	let u = sin(x) * cosh(y);
	let v = cos(x) * sinh(y);
	return vec2(u, v);
}

fn tan_z(x: f32, y: f32) -> vec2<f32> {
	let u = cos(x) * sin(x);
	let v = cosh(y) * sinh(y);
	let phi = pow(cos(x), 2.0) * pow(cosh(y), 2.0) + pow(sin(x), 2.0) * pow(sinh(y), 2.0);
	return vec2(u / phi, v / phi);
}

fn foo(x: f32, y: f32) -> vec2<f32> {
	return vec2(y * x * x, x / 2.0);
}

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let time = uniforms.run_time * 10.0;
    let time2 = mod_n(uniforms.run_time / 10.0, 1.0);
	let power = 4.0 * sin(uniforms.run_time / 4.0);
    let scale = 2.0 * 3.14159; //pow(10.0, 1.0 / power);

    let pos = in.clip_position.xy - vec2(uniforms.width / 2.0, uniforms.height / 2.0);

	var offsets = array(
		vec2<f32>( -0.25, -0.25),
		vec2<f32>( -0.25,  0.0 ),
		vec2<f32>( -0.25,  0.25),
		vec2<f32>(  0.0,  -0.25),
		vec2<f32>(  0.0,   0.0 ),
		vec2<f32>(  0.0,   0.25),
		vec2<f32>(  0.25, -0.25),
		vec2<f32>(  0.25,  0.0 ),
		vec2<f32>(  0.25,  0.25),
	);

	var temp_pos: vec2<f32> = vec2<f32>();
	var z = vec2<f32>();
	var rt = vec2<f32>();
	var uv_total = vec2<f32>();
	for (var i: i32 = 0; i < 9; i++) {
		temp_pos = pos + offsets[i];
		z = (temp_pos / (uniforms.width / 2.0)) * scale;
		rt = xy_to_rt(z.x, z.y);
		// uv_total += z_n(rt.x, rt.y, power);
		uv_total = cos_z(z.x, z.y);
	}

	let uv = uv_total / 9.0;

    let n = 0.05;
    let u = norm_mod_n(uv[0], n);
    let v = norm_mod_n(uv[1], n);

    let u_pow = vec3(2.0, 0.0, 1.0);
    let v_pow = vec3(0.0, 2.0, 1.0);

	let color = vec3(
        pow(u, u_pow[0]) * pow(v, v_pow[0]), 
        pow(u, u_pow[1]) * pow(v, v_pow[1]), 
        pow(u, u_pow[2]) * pow(v, v_pow[2]), 
    );

    let hsv = rgb_to_hsv(color);
	let rgb = hsv_to_rgb(vec3(mod_n(hsv.x + time2, 1.0), hsv.yz));

    return vec4(rgb, 1.0);
}