use bevy::{app::Plugin, ecs::{component::Component, entity::Entity, system::Resource}};

use crate::{common::{Grid, Grid3d}, projection::{CHUNK_SIZE, MAP_SIZE}};

#[derive(Clone, Resource)]
pub struct Map {
    chunks: Grid3d<OverworldChunk>,
}

impl Default for Map {
    fn default() -> Self {
        let chunks = Grid3d::init(MAP_SIZE.0, MAP_SIZE.1, MAP_SIZE.2, OverworldChunk);
        Self {
            chunks,
        }
    }
}

#[derive(Resource, Default)]
pub struct Chunks {
    pub active_idx: usize,
}

#[derive(Clone, Default)]
pub struct OverworldChunk;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<Map>();
        app.init_resource::<Chunks>();
    }
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum ChunkStatus {
    Active,
    Dormant
}

#[derive(Clone, Component)]
pub struct Chunk {
    terrain: Grid<Terrain>,
    pub tiles: Grid<Entity>,
    idx: usize,
}

impl Chunk {
    pub fn new(idx: usize, tiles: Grid<Entity>) -> Self {
        Self {
            terrain: Grid::init(CHUNK_SIZE.0, CHUNK_SIZE.1, Terrain::Grass),
            idx,
            tiles,
        }
    }

    pub fn set_terrain(&mut self, x: usize, y: usize, terrain: Terrain) {
        self.terrain.set(x, y, terrain);
    }

    pub fn get_terrain(&self, x: usize, y: usize) -> Option<&Terrain> {
        self.terrain.get(x, y)
    }

    pub fn idx(&self) -> usize {
        self.idx
    }
}

#[derive(Clone, Copy, Default)]
pub enum Terrain {
    #[default]
    Grass,
}

impl Terrain {
    pub fn sprite_idx(&self) -> usize {
        match self {
            Terrain::Grass => 0,
        }
    }
}
