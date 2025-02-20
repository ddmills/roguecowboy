use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    common::{Grid, Grid3d, Rand}, glyph::Position, player::Player, projection::{chunk_idx, chunk_xyz, CHUNK_SIZE, MAP_SIZE}, GameState
};

use super::{
    LoadChunkEvent, SetChunkStatusEvent, SpawnChunkEvent, UnloadChunkEvent, on_load_chunk,
    on_player_move, on_set_chunk_status, on_spawn_chunk, on_unload_chunk,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<Map>()
            .init_resource::<Chunks>()
            .add_event::<LoadChunkEvent>()
            .add_event::<UnloadChunkEvent>()
            .add_event::<SetChunkStatusEvent>()
            .add_event::<SpawnChunkEvent>()
            .add_systems(
                Update,
                (
                    on_player_move,
                    load_nearby_chunks,
                    on_load_chunk,
                    on_unload_chunk,
                    on_spawn_chunk,
                    on_set_chunk_status,
                    chunk_visibility,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Clone, Resource)]
pub struct Map {
    chunks: Grid3d<OverworldChunk>,
}

impl Default for Map {
    fn default() -> Self {
        let chunks = Grid3d::init(MAP_SIZE.0, MAP_SIZE.1, MAP_SIZE.2, OverworldChunk);
        Self { chunks }
    }
}

pub struct EdgeConstraints {
    pub south: Vec<u8>,
    pub west: Vec<u8>,
}

pub struct ChunkConstraints {
    pub south: Vec<u8>,
    pub west: Vec<u8>,
    pub east: Vec<u8>,
    pub north: Vec<u8>,
}

impl Map {
    fn get_edge_constraints(&self, x: usize, y: usize, z: usize) -> EdgeConstraints {
        if self.chunks.is_oob(x, y, z) {
            return EdgeConstraints {
                south: vec![],
                west: vec![],
            }
        }

        let idx = chunk_idx(x, y, z);
        let mut rand = Rand::seed(idx as u64);

        let mut south = [0; CHUNK_SIZE.0 - 1];
        let mut west = [0; CHUNK_SIZE.1 - 1];

        if y > 0 {
            // river
            if x % 3 == 0 {
                let r = rand.range_n(1, CHUNK_SIZE.0 as i32 - 1) as usize;
                south[r] = 1;
            }
            
            // path
            if x % 4 == 0 {
                let r = rand.range_n(1, CHUNK_SIZE.0 as i32 - 1) as usize;
                south[r] = 2;
            }
        }

        if x > 0 {
            // river
            if y % 2 == 0 {
                let r = rand.range_n(1, CHUNK_SIZE.1 as i32 - 1) as usize;
                west[r] = 1;
            }

            // footpaths
            if y % 2 == 0 {
                let r = rand.range_n(1, CHUNK_SIZE.1 as i32 - 1) as usize;
                west[r] = 2;
            }
        }

        EdgeConstraints {
            south: south.to_vec(),
            west: west.to_vec(),
        }
    }

    pub fn get_chunk_constraints(&self, idx: usize) -> ChunkConstraints {
        let (x, y, z) = chunk_xyz(idx);
        let own = self.get_edge_constraints(x, y, z);

        let east = self.get_edge_constraints(x + 1, y, z);
        let north = self.get_edge_constraints(x, y + 1, z);

        ChunkConstraints {
            north: north.south,
            west: own.west,
            south: own.south,
            east: east.west,
        }
    }
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum ChunkStatus {
    Active,
    Dormant,
}

#[derive(Resource, Default)]
pub struct Chunks {
    pub active: Vec<usize>,
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

    #[inline]
    pub fn to_save(&self) -> ChunkSave {
        ChunkSave {
            idx: self.idx,
            terrain: self.terrain.clone(),
        }
    }

    #[inline]
    pub fn set_terrain(&mut self, x: usize, y: usize, terrain: Terrain) {
        self.terrain.set(x, y, terrain);
    }

    #[inline]
    pub fn get_terrain(&self, x: usize, y: usize) -> Option<&Terrain> {
        self.terrain.get(x, y)
    }

    #[inline]
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
    River,
    Footpath,
}

impl Terrain {
    pub fn sprite_idx(&self) -> usize {
        match self {
            Terrain::Grass => 0,
            Terrain::Dirt => 1,
            Terrain::River => 3,
            Terrain::Footpath => 4,
        }
    }
}

fn chunk_visibility(
    mut cmds: Commands,
    q_player: Query<&Position, With<Player>>,
    q_chunks: Query<(Entity, &Chunk)>,
) {
    let Ok(player) = q_player.get_single() else {
        return;
    };

    for (chunk_e, chunk) in q_chunks.iter() {
        let chunk_pos = chunk_xyz(chunk.idx);

        // compare Z levels
        if chunk_pos.2 != player.z {
            cmds.entity(chunk_e).insert(Visibility::Hidden);
        } else {
            cmds.entity(chunk_e).insert(Visibility::Visible);
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
    q_chunks: Query<(&Chunk, &ChunkStatus)>,
) {
    let mut cur_dormant_chunks = q_chunks
        .iter()
        .filter_map(|(c, s)| match s {
            ChunkStatus::Active => None,
            ChunkStatus::Dormant => Some(c.idx()),
        })
        .collect::<Vec<_>>();

    let mut cur_active_chunks = q_chunks
        .iter()
        .filter_map(|(c, s)| match s {
            ChunkStatus::Active => Some(c.idx()),
            ChunkStatus::Dormant => None,
        })
        .collect::<Vec<_>>();

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

        if z > 0 {
            let above_idx = chunk_idx(x, y, z - 1);
            needed_chunks.push(above_idx);
        }

        if x < MAP_SIZE.0 - 1 {
            let east_idx = chunk_idx(x + 1, y, z);
            needed_chunks.push(east_idx);
        }

        if x > 0 {
            let west_idx = chunk_idx(x - 1, y, z);
            needed_chunks.push(west_idx);
        }

        if z < MAP_SIZE.2 - 1 {
            let below_idx = chunk_idx(x, y, z + 1);
            needed_chunks.push(below_idx);
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

    let chunks_to_unload = [cur_active_chunks, cur_dormant_chunks].concat();

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
