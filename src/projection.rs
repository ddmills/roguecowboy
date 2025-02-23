// width, height, depth, in zones
pub const MAP_SIZE: (usize, usize, usize) = (8, 6, 4);
pub const MAP_SIZE_F32: (f32, f32, f32) = (MAP_SIZE.0 as f32, MAP_SIZE.1 as f32, MAP_SIZE.2 as f32);

// width and height of a zone, in tiles
pub const ZONE_SIZE: (usize, usize) = (40, 20);
pub const ZONE_SIZE_F32: (f32, f32) = (ZONE_SIZE.0 as f32, ZONE_SIZE.1 as f32);

// width and height of a tile, in texels
pub const TILE_SIZE: (usize, usize) = (16, 24);
pub const TILE_SIZE_F32: (f32, f32) = (TILE_SIZE.0 as f32, TILE_SIZE.1 as f32);

// size of a texel, in pixels
pub const TEXEL_SIZE: usize = 1;
pub const TEXEL_SIZE_F32: f32 = TEXEL_SIZE as f32;

pub const Z_LAYER_GROUND: usize = 0;
pub const Z_LAYER_ACTORS: usize = 1;
pub const Z_LAYER_SNAPSHOT: usize = 8;

// Convert a zone position to a zone index
#[inline]
pub fn zone_idx(x: usize, y: usize, z: usize) -> usize {
    x * MAP_SIZE.1 * MAP_SIZE.2 + y * MAP_SIZE.2 + z
}

// Convert a zone index to a zone position
#[inline]
pub fn zone_xyz(zone_idx: usize) -> (usize, usize, usize) {
    (
        zone_idx / (MAP_SIZE.1 * MAP_SIZE.2),
        (zone_idx / MAP_SIZE.2) % MAP_SIZE.1,
        zone_idx % MAP_SIZE.2,
    )
}

// convert world coordinates to local zone coordinates
#[inline]
pub fn world_to_zone_local(x: usize, y: usize) -> (usize, usize) {
    (x % ZONE_SIZE.0, y % ZONE_SIZE.1)
}

// convert local zone coordinates to world coordinates
#[inline]
pub fn zone_local_to_world(zone_idx: usize, x: usize, y: usize) -> (usize, usize, usize) {
    let cpos: (usize, usize, usize) = zone_xyz(zone_idx);

    (cpos.0 * ZONE_SIZE.0 + x, cpos.1 * ZONE_SIZE.1 + y, cpos.2)
}

#[inline]
pub fn world_to_zone_idx(x: usize, y: usize, z: usize) -> usize {
    let cpos = (x / ZONE_SIZE.0, y / ZONE_SIZE.1, z);

    zone_idx(cpos.0, cpos.1, cpos.2)
}

#[inline]
pub fn world_to_px(x: usize, y: usize) -> (usize, usize) {
    (x * TILE_SIZE.0, y * TILE_SIZE.1)
}

pub fn zone_transform_center(zone_idx: usize) -> (f32, f32) {
    let zone_pos = zone_xyz(zone_idx);
    (
        (zone_pos.0 * ZONE_SIZE.0 * TILE_SIZE.0) as f32
            + ((ZONE_SIZE_F32.0 * TILE_SIZE_F32.0) / 2.)
            - (TILE_SIZE_F32.0 / 2.),
        (zone_pos.1 * ZONE_SIZE.1 * TILE_SIZE.1) as f32
            + ((ZONE_SIZE_F32.1 * TILE_SIZE_F32.1) / 2.)
            - (TILE_SIZE_F32.1 / 2.),
    )
}

// returns true if world coordinate is in bounds
pub fn is_in_bounds(x: u32, y: u32, z: u32) -> bool {
    x > 0
        && y > 0
        && z > 0
        && x < (MAP_SIZE.0 * ZONE_SIZE.0) as u32
        && y < (MAP_SIZE.1 * ZONE_SIZE.1) as u32
        && z < MAP_SIZE.2 as u32
}
