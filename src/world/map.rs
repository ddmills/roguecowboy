use crate::common::{Grid, Grid3d};

pub const MAP_SIZE: (usize, usize, usize) = (8, 6, 10);

#[derive(Clone)]
pub struct Map {
    chunks: Grid3d<Chunk>,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            chunks: Grid3d::init(MAP_SIZE.0, MAP_SIZE.1, MAP_SIZE.2, Chunk::default()),
        }
    }
}

#[derive(Clone)]
pub struct Chunk {
    terrain: Grid<Terrain>
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            terrain: Grid::init(80, 60, Terrain::Grass),
        }
    }
}

impl Chunk {
    pub fn set_terrain(&mut self, x: usize, y: usize, terrain: Terrain) {
        self.terrain.set(x, y, terrain);
    }
}

#[derive(Clone, Copy, Default)]
pub enum Terrain {
    #[default]
    Grass,
}
