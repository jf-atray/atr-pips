enable f16;

@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@group(1) @binding(0)
var tex: texture_2d<f32>;

@group(1) @binding(1)
var sam: sampler;

struct Material {
    natural_scale: vec3<f32>,
}

@group(1) @binding(2)
var<uniform> material: Material;

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    // mesh vertex
    @location(0) local: vec3<f32>,
    @location(1) uv: vec2<f32>,

    // instance data
    @location(2) pos: vec3<f32>,
    @location(3) rot: vec4<f16>,
    @location(4) scale: vec3<f16>,
    @location(5) color: vec4<f16>,
) -> VertexOut {
    
    let q = normalize(vec4<f32>(rot));

    let scaled_local =
        local *
        material.natural_scale *
        vec3<f32>(scale);

    let world = pos + (rotate_vec3(scaled_local, q));

    var out: VertexOut;
    out.clip_position = view_proj * vec4<f32>(world, 1.0);
    out.uv = uv;
    out.color = vec4<f32>(color);
    return out;
}

//there should be an in-build fn somewehre
fn rotate_vec3(v: vec3<f32>, q: vec4<f32>) -> vec3<f32> {
    let t = 2.0 * cross(q.xyz, v);
    return v + q.w * t + cross(q.xyz, t);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(tex, sam, in.uv) * in.color;
}