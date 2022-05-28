struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> view: View;

struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] sprite_index: i32;
    [[location(2)]] color: vec4<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

struct TilemapGpuData {
    transform: mat4x4<f32>;
};

[[group(2), binding(0)]]
var<uniform> tilemap: TilemapGpuData;

[[stage(vertex)]]
fn vertex(
    [[builtin(vertex_index)]] vertex_index: u32,
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>,
    [[location(2)]] sprite_index: i32,
    [[location(3)]] vertex_color: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.sprite_index = sprite_index;
    out.position = view.view_proj * tilemap.transform * vec4<f32>(vertex_position, 1.0);
    out.color = vec4<f32>((vec4<u32>(vertex_color) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(255u)) / 255.0;

    return out;
}

[[group(1), binding(0)]]
var sprite_texture: texture_2d_array<f32>;
[[group(1), binding(1)]]
var sprite_sampler: sampler;

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var color = textureSample(sprite_texture, sprite_sampler, in.uv, in.sprite_index);
    color = in.color * color;
    return color;
}
