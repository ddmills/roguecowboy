use crate::{
    common::{Grid, Perlin, Rand},
    projection::ZONE_SIZE,
    world::Terrain,
};

use super::{TileSnapColor, ZoneConstraints, ZoneSnapshot};

pub fn edge_snapshot(constraints: &ZoneConstraints) -> ZoneSnapshot {
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

    ZoneSnapshot { data }
}

pub fn grayscale_snapshot(g: &Grid<f32>) -> ZoneSnapshot {
    let data = g.map(|_, _, v| TileSnapColor::gray(*v));

    ZoneSnapshot { data }
}

pub fn bool_snapshot(g: &Grid<bool>) -> ZoneSnapshot {
    let data = g.map(|_, _, v| match v {
        true => TileSnapColor::White,
        false => TileSnapColor::Black,
    });

    ZoneSnapshot { data }
}

pub fn edge_gradient_buffer(buffer: usize, pow: f32) -> Grid<f32> {
    let mut g = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, 1.);

    for x in 0..ZONE_SIZE.0 {
        for z in 0..buffer {
            let v = (z as f32 / buffer as f32).powf(pow);

            if z < x && z < (ZONE_SIZE.0 - x) {
                g.set(x, z, v);
                g.set(x, ZONE_SIZE.1 - z - 1, v);
            }
        }
    }

    for y in 0..ZONE_SIZE.1 {
        for z in 0..buffer {
            let v = (z as f32 / buffer as f32).powf(pow);

            if z <= y && z < (ZONE_SIZE.1 - y) {
                g.set(z, y, v);
                g.set(ZONE_SIZE.0 - z - 1, y, v);
            }
        }
    }

    g
}

pub fn noise_grid(seed: u32, frequency: f32, octaves: u32, lacunarity: f32) -> Grid<f32> {
    let mut nz = Perlin::new(seed, frequency, octaves, lacunarity);

    Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |x, y| nz.get(x as f32, y as f32))
}

pub fn rand_grid(seed: u32) -> Grid<bool> {
    let mut rand = Rand::seed(seed as u64);

    Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| rand.bool(0.5))
}

pub fn terrain_snapshot(t: &Grid<Terrain>) -> ZoneSnapshot {
    let mut data = Grid::init(ZONE_SIZE.0, ZONE_SIZE.1, TileSnapColor::White);

    for x in 0..t.width() {
        for y in 0..t.height() {
            if let Some(v) = t.get(x, y) {
                data.set(x, y, TileSnapColor::for_terrain(*v));
            }
        }
    }

    ZoneSnapshot { data }
}
