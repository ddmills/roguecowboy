use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{common::{Grid, Grid3d}, projection::{chunk_idx, chunk_xyz, MAP_SIZE}, GameState};

use super::{on_load_chunk, on_set_chunk_status, on_spawn_chunk, on_unload_chunk, LoadChunkEvent, SetChunkStatusEvent, SpawnChunkEvent, UnloadChunkEvent};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<Map>()
            .init_resource::<Chunks>()
            .add_event::<LoadChunkEvent>()
            .add_event::<UnloadChunkEvent>()
            .add_event::<SetChunkStatusEvent>()
            .add_event::<SpawnChunkEvent>()
            .add_systems(Update, (
                load_nearby_chunks,
                on_load_chunk,
                on_unload_chunk,
                on_spawn_chunk,
                on_set_chunk_status,
            ).chain().run_if(in_state(GameState::Playing)));
    }
}

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

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum ChunkStatus {
    Active,
    Dormant
}

#[derive(Resource, Default)]
pub struct Chunks {
    pub active: Vec<usize>,
    pub dormant: Vec<usize>,
}

#[derive(Clone, Component)]
pub struct Chunk {
    terrain: Grid<Terrain>,
    pub tiles: Grid<Entity>,
    idx: usize,
}

impl Chunk {
    pub fn new(idx: usize, terrain: Grid<Terrain>, tiles: Grid<Entity>) -> Self {
        Self {
            terrain,
            idx,
            tiles,
        }
    }

    pub fn to_save(&self) -> ChunkSave {
        ChunkSave {
            idx: self.idx,
            terrain: self.terrain.clone(),
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

#[derive(Deserialize, Serialize)]
pub struct ChunkSave {
    pub idx: usize,
    pub terrain: Grid<Terrain>,
}

#[derive(Clone, Default)]
pub struct OverworldChunk;

#[derive(Clone, Copy, Default, Deserialize, Serialize)]
pub enum Terrain {
    #[default]
    Grass,
    Dirt,
}

impl Terrain {
    pub fn sprite_idx(&self) -> usize {
        match self {
            Terrain::Grass => 0,
            Terrain::Dirt => 1,
        }
    }
}

// determine which chunks should
//  - be loaded
//  - be unloaded
//  - change status
fn load_nearby_chunks(
    chunks: Res<Chunks>,
    mut e_load_chunk: EventWriter<LoadChunkEvent>,
    mut e_unload_chunk: EventWriter<UnloadChunkEvent>,
    mut e_set_chunk_status: EventWriter<SetChunkStatusEvent>,
    q_chunks: Query<(&Chunk, &ChunkStatus)>
) {
    let mut cur_dormant_chunks = q_chunks.iter().filter_map(|(c, s)| {
        match s {
            ChunkStatus::Active => None,
            ChunkStatus::Dormant => Some(c.idx()),
        }
    }).collect::<Vec<_>>();

    let mut cur_active_chunks = q_chunks.iter().filter_map(|(c, s)| {
        match s {
            ChunkStatus::Active => Some(c.idx()),
            ChunkStatus::Dormant => None,
        }
    }).collect::<Vec<_>>();

    let mut needed_chunks = chunks.active.clone();

    for idx in chunks.active.iter() {
        let (x, y, z) = chunk_xyz(*idx);

        if y < MAP_SIZE.1 - 1 {
            let north_idx = chunk_idx(x, y + 1, z);
            needed_chunks.push(north_idx);
    
            if x < MAP_SIZE.0 - 1 {
                let north_east_idx = chunk_idx(x + 1, y + 1, z);
                needed_chunks.push(north_east_idx);
            }
    
            if x > 0 {
                let north_west_idx = chunk_idx(x - 1, y + 1, z);
                needed_chunks.push(north_west_idx);
            }
        }
    
        if y > 0 {
            let south_idx = chunk_idx(x, y - 1, z);
            needed_chunks.push(south_idx);
    
            if x < MAP_SIZE.0 - 1 {
                let south_east_idx = chunk_idx(x + 1, y - 1, z);
                needed_chunks.push(south_east_idx);
            }
    
            if x > 0 {
                let south_west_idx = chunk_idx(x - 1, y - 1, z);
                needed_chunks.push(south_west_idx);
            }
        }
    
        if x < MAP_SIZE.0 - 1 {
            let east_idx = chunk_idx(x + 1, y, z);
            needed_chunks.push(east_idx);
        }
    
        if x > 0 {
            let west_idx = chunk_idx(x - 1, y, z);
            needed_chunks.push(west_idx);
        }
    }

    let mut chunks_to_load = vec![];
    let mut chunks_to_dormant = vec![];
    let mut chunks_to_active = vec![];

    needed_chunks.sort();
    needed_chunks.dedup();

    for idx in needed_chunks.iter() {
        let is_active = chunks.active.contains(idx);

        if let Some(cur_pos) = cur_active_chunks.iter().position(|&i| i == *idx) {
            cur_active_chunks.swap_remove(cur_pos);

            // chunk is active, but needs to be dormant.
            if !is_active {
                chunks_to_dormant.push(*idx);
            }
        } else if let Some(cur_pos) = cur_dormant_chunks.iter().position(|&i| i == *idx) {
            cur_dormant_chunks.swap_remove(cur_pos);
            
            // chunk is dormant but needs to be active
            if is_active {
                chunks_to_active.push(*idx);
            }
        } else {
            // chunk is not dormant or active, but needed. We must load it
            chunks_to_load.push(*idx);

            // also needs to be active
            if is_active {
                chunks_to_active.push(*idx);
            }
        }
    }

    let mut chunks_to_unload = [cur_active_chunks, cur_dormant_chunks].concat();
    
    // chunks_to_unload.sort();
    // chunks_to_unload.dedup();

    // chunks_to_load.sort();
    // chunks_to_load.dedup();

    // chunks_to_active.sort();
    // chunks_to_active.dedup();

    // chunks_to_dormant.sort();
    // chunks_to_dormant.dedup();

    for idx in chunks_to_load.iter() {
        e_load_chunk.send(LoadChunkEvent(*idx));
    }

    for idx in chunks_to_unload.iter() {
        e_unload_chunk.send(UnloadChunkEvent(*idx));
    }

    for idx in chunks_to_active.iter() {
        e_set_chunk_status.send(SetChunkStatusEvent {
            idx: *idx,
            status: ChunkStatus::Active,
        });
    }

    for idx in chunks_to_dormant.iter() {
        e_set_chunk_status.send(SetChunkStatusEvent {
            idx: *idx,
            status: ChunkStatus::Dormant,
        });
    }
}
