@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@group(1) @binding(0)
var tex: texture_2d<f32>;

@group(1) @binding(1)
var sam: sampler;

struct Material {
    natural_scale: vec2<f32>,
    color: vec4<f32>,
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
    @location(0) local: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) pos: vec3<f32>,
    @location(3) rot: vec4<f32>,
    @location(4) scale: vec2<f32>,
    @location(5) color: vec4<f32>,
) -> VertexOut {
    let natural = material.natural_scale * scale;
    let offset2 = local * 0.5 * natural;
    let offset = vec3<f32>(offset2, 0.0);

    let q = rot;
    let x = q.x;
    let y = q.y;
    let z = q.z;
    let w = q.w;

    let xx = 2.0 * x * x;
    let yy = 2.0 * y * y;
    let zz = 2.0 * z * z;
    let xy = 2.0 * x * y;
    let xz = 2.0 * x * z;
    let yz = 2.0 * y * z;
    let wx = 2.0 * w * x;
    let wy = 2.0 * w * y;
    let wz = 2.0 * w * z;

    let rotation = mat3x3<f32>(
        vec3<f32>(1.0 - yy - zz, xy + wz, xz - wy),
        vec3<f32>(xy - wz, 1.0 - xx - zz, yz + wx),
        vec3<f32>(xz + wy, yz - wx, 1.0 - xx - yy),
    );

    let world = pos + (rotation * offset);

    var out: VertexOut;
    out.clip_position = view_proj * vec4<f32>(world, 1.0);
    out.uv = uv;
    out.color = material.color * color;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(tex, sam, in.uv) * in.color;
}
