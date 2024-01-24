struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) colour: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) colour: vec3<f32>,
};

struct Mouse {
    x: f32,                 // offset(0)  align(4)  size(4)
    y: f32,                 // offset(4)  align(4)  size(4)
    drag: i32,              // offset(8)  align(4)  size(4)
    px: f32,                // offset(12) align(4)  size(4)
    py: f32,                // offset(16) align(4)  size(4)
    centre_x: f32,          // offset(20) align(4)  size(4)
    centre_y: f32,          // offset(24) align(4)  size(4)
}; 
 
struct ShaderUniform {      //            align(16) size(72)
    resolution: vec2<f32>,  // offset(0)  align(8)  size(8)
    iterations: i32,        // offset(8)  align(4)  size(4)
    value: f32,             // offset(12) align(4)  size(4)
    mouse: Mouse,           // offset(16) align(16) size(32)
    @align(16) axis_range: f32,        // offset(48) align(4)  size(4)
    exponent: f32,          // offset(52) align(4)  size(4)
    // -- implicit padding  // offset(56)           size(8)
};

@group(0) @binding(0)
var<uniform> shader_info: ShaderUniform; 

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(model.position,1.0);
    out.colour = model.colour;
    return out;
}

// Mandelbrot Shader Code
// Equation: z = z^2 + c

fn mandelbrot(coord: vec2<f32>) -> f32 {
    var z: vec2<f32> = vec2<f32>(0.0, 0.0);
    var iteration: i32 = 0;
    loop {
        if (length(z) >= 4.0 || iteration >= shader_info.iterations) { break; }
        z = vec2<f32>(pow(abs(z.x), shader_info.exponent) - pow(abs(z.y), shader_info.exponent), shader_info.value * z.x * z.y) + coord;
        iteration += 1;
    }
    if (iteration == shader_info.iterations) { 
        return f32(shader_info.iterations); 
    }
    return f32(iteration) + 1.0 - log2(log2(length(z)));
}

fn get_coordinate(fs_coord: vec4<f32>) -> vec2<f32> {
    var aspect_ratio = shader_info.resolution.x / shader_info.resolution.y;
    var normalised_coords: vec2<f32> = fs_coord.xy / shader_info.resolution.xy;

    var minx = shader_info.mouse.centre_x - shader_info.axis_range/2.0 * aspect_ratio;
    var maxx = shader_info.mouse.centre_x + shader_info.axis_range/2.0 * aspect_ratio;
    var miny = shader_info.mouse.centre_y - shader_info.axis_range/2.0;
    var maxy = shader_info.mouse.centre_y + shader_info.axis_range/2.0;
    
    normalised_coords.x = minx + normalised_coords.x * (maxx - minx);
    normalised_coords.y = miny + normalised_coords.y * (maxy - miny);

    return normalised_coords;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var coord: vec2<f32> = get_coordinate(in.position);
    var iterations = mandelbrot(coord);
    var shade = 0.0;
    if iterations != f32(shader_info.iterations) { shade = iterations / f32(shader_info.iterations); }
    return vec4<f32>(shade);
}