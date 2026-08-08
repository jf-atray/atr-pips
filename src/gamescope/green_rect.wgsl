@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> color: vec4<f32>;

@vertex
fn vs_main(
    @location(0) local: vec2<f32>,
    @location(1) pos: vec3<f32>,
    @location(2) _rot: vec4<f32>,
) -> @builtin(position) vec4<f32> {
    let offset = vec3<f32>(local * 0.5, 0.0);
    let world = pos + offset;
    return view_proj * vec4<f32>(world, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return color;
}
