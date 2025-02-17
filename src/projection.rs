use bevy::log::info;


// width, height, depth, in chunks
pub const MAP_SIZE: (usize, usize, usize) = (8, 6, 4);

// width and height of a chunk, in tiles
pub const CHUNK_SIZE: (usize, usize) = (12, 8);

// width and height of a tile, in pixels
pub const TILE_SIZE: (usize, usize) = (16, 16);

pub const Z_LAYER_GROUND: usize = 0;
pub const Z_LAYER_ACTORS: usize = 1;

// Convert a chunk position to a chunk index
#[inline]
pub fn chunk_idx(x: usize, y: usize, z: usize) -> usize {
    x * MAP_SIZE.1 * MAP_SIZE.2 + y * MAP_SIZE.2 + z
}

// Convert a chunk index to a chunk position
#[inline]
pub fn chunk_xyz(chunk_idx: usize) -> (usize, usize, usize) {
    (chunk_idx / (MAP_SIZE.1 * MAP_SIZE.2), (chunk_idx / MAP_SIZE.2) % MAP_SIZE.1, chunk_idx % MAP_SIZE.2)
}

// convert world coordinates to local chunk coordinates
#[inline]
pub fn world_to_chunk_local(x: usize, y: usize, z: usize) -> (usize, usize, usize)
{
    (
        x % CHUNK_SIZE.0,
        y % CHUNK_SIZE.1,
        z,
    )
}

// convert local chunk coordinates to world coordinates
#[inline]
pub fn chunk_local_to_world(chunk_idx: usize, x: usize, y: usize) -> (usize, usize, usize)
{
    let cpos: (usize, usize, usize) = chunk_xyz(chunk_idx);

    (
        cpos.0 * CHUNK_SIZE.0 + x,
        cpos.1 * CHUNK_SIZE.1 + y,
        cpos.2,
    )
}

#[inline]
pub fn world_to_chunk_idx(x: usize, y: usize, z: usize) -> usize {
    let cpos = (
        x / CHUNK_SIZE.0,
        y / CHUNK_SIZE.1,
        z
    );

    chunk_idx(cpos.0, cpos.1, cpos.2)
}

#[inline]
pub fn world_to_px(x: usize, y: usize) -> (usize, usize)
{
    (x * TILE_SIZE.0, y * TILE_SIZE.1)
}

pub fn world_in_bounds(x: u32, y: u32, z: u32) -> bool {
    x > 0
        && y > 0
        && z > 0
        && x < (MAP_SIZE.0 * CHUNK_SIZE.0) as u32
        && y < (MAP_SIZE.1 * CHUNK_SIZE.1) as u32
        && z < MAP_SIZE.2 as u32
}
