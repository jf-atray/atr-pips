enable f16;

@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@group(1) @binding(0)
var tex: texture_2d<f32>;

@group(1) @binding(1)
var sam: sampler;

struct Material {
    natural_scale: vec3<f32>,
    y_offset: f32,
}

@group(1) @binding(2)
var<uniform> material: Material;

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

fn unpack_affine(m0: vec4<f16>, m1: vec4<f16>, m2: vec4<f16>) -> mat4x3<f32> {
    let c0 = vec3<f32>(f32(m0.x), f32(m0.y), f32(m0.z));
    let c1 = vec3<f32>(f32(m0.w), f32(m1.x), f32(m1.y));
    let c2 = vec3<f32>(f32(m1.z), f32(m1.w), f32(m2.x));
    let c3 = vec3<f32>(f32(m2.y), f32(m2.z), f32(m2.w));
    return mat4x3<f32>(c0, c1, c2, c3);
}

@vertex
fn vs_main(
    // mesh vertex
    @location(0) local: vec3<f32>,
    @location(1) uv: vec2<f32>,

    // instance data: packed mat4x3<f16> + color
    @location(2) m0: vec4<f16>,
    @location(3) m1: vec4<f16>,
    @location(4) m2: vec4<f16>,
    @location(5) color: vec4<f16>,
) -> VertexOut {
    let matrix = unpack_affine(m0, m1, m2);

    let scaled_local = local * material.natural_scale;
    let offset_local = scaled_local + vec3<f32>(0.0, material.y_offset, 0.0);

    let world = (matrix * vec4<f32>(offset_local, 1.0)).xyz;

    var out: VertexOut;
    out.clip_position = view_proj * vec4<f32>(world, 1.0);
    out.uv = uv;
    out.color = vec4<f32>(f32(color.x), f32(color.y), f32(color.z), f32(color.w));
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(tex, sam, in.uv) * in.color;
}