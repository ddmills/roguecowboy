use bevy::prelude::*;

use crate::{common::{Grid, Rand}, glyph::{Glyph, Position}, player::{Player, PlayerMovedEvent}, projection::{chunk_local_to_world, chunk_xyz, world_to_chunk_idx, CHUNK_SIZE, Z_LAYER_GROUND}, save::{save_chunk, try_load_chunk}};

use super::{Chunk, ChunkSave, ChunkStatus, Chunks, Terrain};

#[derive(Event)]
pub struct LoadChunkEvent(pub usize);

#[derive(Event)]
pub struct UnloadChunkEvent(pub usize);

#[derive(Event)]
pub struct SetChunkStatusEvent {
    pub idx: usize,
    pub status: ChunkStatus,
}

#[derive(Event)]
pub struct SpawnChunkEvent {
    pub data: ChunkSave,
}

pub fn on_load_chunk(mut e_load_chunk: EventReader<LoadChunkEvent>, mut e_spawn_chunk: EventWriter<SpawnChunkEvent>) {
    for LoadChunkEvent(chunk_idx) in e_load_chunk.read() {
        info!("load chunk! {}", chunk_idx);

        if let Some(save_data) = try_load_chunk(*chunk_idx) {
            e_spawn_chunk.send(SpawnChunkEvent { data: save_data });
            continue;
        };

        let mut r = Rand::seed(*chunk_idx as u64);
        let terrains = vec![Terrain::Grass, Terrain::Dirt];

        let terrain = Grid::init_fill(CHUNK_SIZE.0, CHUNK_SIZE.1, |x, y| {
            r.pick(&terrains)
        });

        let data = ChunkSave {
            idx: *chunk_idx,
            terrain,
        };

        e_spawn_chunk.send(SpawnChunkEvent { data });
    }
}

pub fn on_unload_chunk(mut e_unload_chunk: EventReader<UnloadChunkEvent>, mut cmds: Commands, q_chunks: Query<(Entity, &Chunk)>) {
    for UnloadChunkEvent(chunk_idx) in e_unload_chunk.read() {
        info!("unload chunk! {}", chunk_idx);

        let Some((chunk_e, chunk)) = q_chunks.iter().find(|(_, c)| c.idx() == *chunk_idx) else {
            continue;
        };

        save_chunk(&chunk.to_save());

        cmds.entity(chunk_e).despawn_recursive();
    }
}

pub fn on_spawn_chunk(mut e_spawn_chunk: EventReader<SpawnChunkEvent>, mut cmds: Commands)
{
    for e in e_spawn_chunk.read() {
        info!("spawn chunk! {}", e.data.idx);
        let chunk_e = cmds.spawn((
            Name::new(format!("chunk-{}", e.data.idx)),
            Transform::default(),
            Visibility::Hidden,
            ChunkStatus::Dormant,
        )).id();

        let mut tiles = vec![];

        for x in 0..CHUNK_SIZE.0 {
            for y in 0..CHUNK_SIZE.1 {
                let terrain = e.data.terrain.get(x, y).unwrap();
                let wpos = chunk_local_to_world(e.data.idx, x, y);

                let tile_id = cmds.spawn((
                    Glyph {
                        idx: terrain.sprite_idx(),
                        fg: Color::srgb_u8(35, 37, 37),
                        bg: Color::srgb_u8(0, 0, 0)
                    },
                    Position::new(wpos.0, wpos.1, wpos.2, Z_LAYER_GROUND),
                    ChunkStatus::Dormant,
                )).set_parent(chunk_e).id();

                tiles.push(tile_id);
            }
        }

        let tile_grid = Grid::init_from_vec(CHUNK_SIZE.0, CHUNK_SIZE.1, tiles);
        let chunk = Chunk::new(e.data.idx, e.data.terrain.clone(), tile_grid);

        cmds.entity(chunk_e).insert(chunk);
    }
}

pub fn on_set_chunk_status(mut e_set_chunk_status: EventReader<SetChunkStatusEvent>, mut cmds: Commands, q_chunks: Query<(Entity, &Chunk)>, q_player: Query<&Position, With<Player>>)
{
    for e in e_set_chunk_status.read() {        
        let Some((chunk_e, chunk)) = q_chunks.iter().find(|(en, c)| c.idx() == e.idx) else {
            continue;
        };

        let Ok(player) = q_player.get_single() else {
            continue;
        };

        cmds.entity(chunk_e).insert(e.status);

        for tile in chunk.tiles.iter() {
            cmds.entity(*tile).insert(e.status);
        }
    }
}

// check when player moves to a different chunk and set it as active
pub fn on_player_move(
    mut e_player_moved: EventReader<PlayerMovedEvent>,
    // q_player: Query<&Position, With<Player>>,
    mut chunks: ResMut<Chunks>
) {
    for e in e_player_moved.read() {
        // let player = q_player.single();
        let player_chunk_idx = world_to_chunk_idx(e.x, e.y, e.z);
        
        if !chunks.active.contains(&player_chunk_idx) {
            chunks.active = vec![player_chunk_idx];
        }
    }
}
