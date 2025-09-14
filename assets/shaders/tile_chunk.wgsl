#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
}

@group(2) @binding(100) var<uniform> chunk_radius: u32;
@group(2) @binding(101) var<uniform> tile_size: f32;
@group(2) @binding(102) var<storage, read> tiles: array<i32>;

const DEEP_WATER: i32 = 0;
const WATER: i32 = 1;
const SAND: i32 = 2;
const GRASSLAND: i32 = 3;
const HILLS: i32 = 4;
const MOUNTAINS: i32 = 5;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    var pos = in.world_position.xz;
    let tile = world_to_tile(pos);
    let chunk_center = tile_to_center(tile);
    let tile_offset = tile - chunk_center;
    let index = tile_to_index(tile_offset);
    let kind = tiles[index];
    pbr_input.material.base_color = tile_kind_to_color(kind);

    if (kind == DEEP_WATER || kind == WATER) {
        let time = globals.time;
        let uv = in.uv * 5.0;
        let pos = in.world_position.xy;

        let offset = hash(floor(pos));
        let n = noise(uv + vec2(time * 0.5 + offset, time * 0.3 + offset));
        let n2 = noise(uv * 2.0 + vec2(-time * 0.7, time * 0.4) + offset);
        let ripple = (n + 0.5 * n2) * 0.05;

        pbr_input.material.base_color += ripple;
    }

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}

// 2D value noise helper
fn hash(p: vec2<f32>) -> f32 {
    // A simple but effective pseudo-random hash function
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    // Four corners in 2D of our cell
    let a = hash(i);
    let b = hash(i + vec2(1.0, 0.0));
    let c = hash(i + vec2(0.0, 1.0));
    let d = hash(i + vec2(1.0, 1.0));

    // Smooth interpolation (fade function)
    let u = f * f * (3.0 - 2.0 * f);

    // Bilinear interpolate the four corners
    return mix(a, b, u.x) +
           (c - a)* u.y * (1.0 - u.x) +
           (d - b) * u.x * u.y;
}

fn tile_kind_to_color(kind: i32) -> vec4<f32> {
    if (kind == DEEP_WATER) {
        return vec4<f32>(0.0, 0.18, 0.35, 1.0);
    } else if (kind == WATER) {
        return vec4<f32>(0.0, 0.3, 0.5, 1.0);
    } else if (kind == SAND) {
        return vec4<f32>(0.85, 0.73, 0.5, 1.0);
    } else if (kind == GRASSLAND) {
        return vec4<f32>(0.4, 0.65, 0.3, 1.0);
    } else if (kind == HILLS) {
        return vec4<f32>(0.45, 0.4, 0.35, 1.0);
    } else if (kind == MOUNTAINS) {
        return vec4<f32>(0.45, 0.45, 0.45, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}

fn world_to_tile(pos: vec2<f32>) -> vec2<i32> {
    return vec2<i32>(floor((pos + tile_size / 2.0) / tile_size));
}

fn tile_to_index(tile: vec2<i32>) -> u32 {
    let diameter = i32(chunk_radius) * 2 + 1;
    let x = tile.x + i32(chunk_radius);
    let y = tile.y + i32(chunk_radius);
    return u32(y * diameter + x);
}

fn tile_to_center(tile: vec2<i32>) -> vec2<i32> {
    let diameter = i32(chunk_radius) * 2 + 1;

    return (tile + sign(tile) * i32(chunk_radius)) / diameter * diameter;
}
