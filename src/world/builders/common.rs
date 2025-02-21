use crate::{common::Grid, projection::ZONE_SIZE, world::Terrain};

use super::{TileSnapColor, ZoneConstraints, ZoneSnapshot};

pub fn edge_snapshot(constraints: &ZoneConstraints) -> ZoneSnapshot
{
    let mut data = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, TileSnapColor::White);

    for (x, v) in constraints.north.iter().enumerate() {
        data.set(x, ZONE_SIZE.1 - 1, TileSnapColor::for_edge(*v));
    }

    for (x, v) in constraints.south.iter().enumerate() {
        data.set(x, 0, TileSnapColor::for_edge(*v));
    }

    for (y, v) in constraints.east.iter().enumerate() {
        data.set(ZONE_SIZE.0 - 1, y, TileSnapColor::for_edge(*v));
    }

    for (y, v) in constraints.west.iter().enumerate() {
        data.set(0, y, TileSnapColor::for_edge(*v));
    }

    ZoneSnapshot {
        data
    }
}

pub fn grayscale_snapshot(g: &Grid<f32>) -> ZoneSnapshot
{
    let data = g.map(|_, _, v| TileSnapColor::gray(*v));

    ZoneSnapshot {
        data,
    }
}

pub fn edge_gradient_buffer(buffer: usize) -> Grid<f32>
{
    let mut g = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, 0.);

    for x in 0..ZONE_SIZE.0 {
        for z in 0..buffer {
            let v = 1. - (z as f32 / buffer as f32);

            if z < x && z < (ZONE_SIZE.0 - x) {
                g.set(x, z, v);
                g.set(x, ZONE_SIZE.1 - z - 1, v);
            }
        }
    }

    for y in 0..ZONE_SIZE.1 {
        for z in 0..buffer {
            let v = 1. - (z as f32 / buffer as f32);

            if z <= y && z < (ZONE_SIZE.1 - y) {
                g.set(z, y, v);
                g.set(ZONE_SIZE.0 - z - 1, y, v);
            }
        }
    }

    g
}

pub fn terrain_snapshot(t: &Grid<Terrain>) -> ZoneSnapshot
{
    let mut data = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, TileSnapColor::White);

    for x in 0..t.width() {
        for y in 0..t.height() {
            if let Some(v) = t.get(x, y) {
                data.set(x, y, TileSnapColor::for_terrain(*v));
            }
        }
    }

    ZoneSnapshot {
        data,
    }
}
