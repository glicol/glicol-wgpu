// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// @group(1) @binding(0)
// var<uniform> position: f32;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    // by default it's red, but we use this to get binary data
    var grayscale = 1. * color.r + 1. * color.g + 1. * color.b;
    var fontcolor = grayscale * 0.5;
    if (in.tex_coords.x == 0.0 || in.tex_coords.y == 0.0) {
        return vec4<f32>(0.0, 0.3, 0.5, 0.9);
    } else {
        return vec4<f32>(fontcolor, fontcolor, fontcolor, color.a);
    }
    
}