#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> fg1: vec4<f32>;
@group(2) @binding(1) var<uniform> fg2: vec4<f32>;
@group(2) @binding(2) var<uniform> bg: vec4<f32>;
@group(2) @binding(3) var<uniform> outline: vec4<f32>;
@group(2) @binding(4) var<uniform> idx: u32;
@group(2) @binding(5) var atlas_texture: texture_2d<f32>;
@group(2) @binding(6) var atlas_texture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv_scaled = mesh.uv / 16.0; // atlas is 16x16
    let uv_offset = vec2(f32(idx % 16), f32(idx / 16)) / 16.;
    let uv = uv_offset + uv_scaled;

    let tex = textureSample(atlas_texture, atlas_texture_sampler, uv);

    var color = vec4(1.0, 0.1, 0.1, 1.0);

    // transparent (background)
    if (tex.a == 0) {
        return bg;
    }

    // black (primary)
    if (tex.r == 0 && tex.g == 0 && tex.b == 0) {
        return fg1;
    }

    // white (secondary)
    if (tex.r == 1 && tex.g == 1 && tex.b == 1) {
        return fg2;
    }

    // red (outline)
    if (tex.r == 1 && tex.g == 0 && tex.b == 0) {
        return outline;
    }

    return fg1;
    // return vec4(1.0, 0.0, 1.0, 1.0);
}
