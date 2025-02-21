use crate::{common::Grid, projection::ZONE_SIZE, world::Terrain};

use super::{SnapshotScheme, TileSnapColor, ZoneConstraints, ZoneSnapshot};

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
        scheme: SnapshotScheme::Edges,
        data
    }
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
        scheme: SnapshotScheme::Terrain,
        data,
    }
}
