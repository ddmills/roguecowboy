use bevy::prelude::*;
use camera::CameraPlugin;
use common::Grid;
use glyph::{on_status_change, setup_tileset, update_glyph_sprite, update_positions, Glyph, Position, Tileset};
use player::{on_player_move, player_input, setup_player};
use projection::{chunk_idx, chunk_local_to_world, chunk_xyz, CHUNK_SIZE, MAP_SIZE, Z_LAYER_GROUND};
use world::{Chunk, ChunkStatus, Chunks, MapPlugin};

mod common;
mod camera;
mod world;
mod player;
mod glyph;
mod projection;

#[derive(Default, States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

pub fn go_to_state(state:GameState) -> impl Fn(ResMut<NextState<GameState>>) {
    move |mut next| {
        next.set(state);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(CameraPlugin)
        .add_plugins(MapPlugin)
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::srgb_u8(19, 27, 37)))
        .init_resource::<Tileset>()
        .add_event::<LoadChunkEvent>()
        .add_event::<UnloadChunkEvent>()
        .add_systems(OnEnter(GameState::Loading), (setup_tileset, go_to_state(GameState::Playing)).chain())
        .add_systems(OnEnter(GameState::Playing), (setup_player).chain())
        .add_systems(Update, (player_input, on_player_move, on_load_chunk, on_unload_chunk).chain().run_if(in_state(GameState::Playing)))
        .add_systems(Update, (update_glyph_sprite, update_positions, on_status_change))
        .add_systems(Update, load_nearby_chunks.run_if(in_state(GameState::Playing).and(resource_changed::<Chunks>)))
        .run();
}

#[derive(Event)]
struct LoadChunkEvent {
    pub idx: usize,
    pub is_active: bool,
}

impl LoadChunkEvent {
    pub fn new(idx:usize) -> Self {
        Self {
            idx,
            is_active: false,
        }
    }
}

#[derive(Event)]
struct UnloadChunkEvent {
    pub idx: usize,
}

fn load_nearby_chunks(
    chunks: Res<Chunks>,
    mut e_load_chunk: EventWriter<LoadChunkEvent>,
    mut e_unload_chunk: EventWriter<UnloadChunkEvent>,
    q_chunks: Query<&Chunk>
) {
    info!("chunk changed {}", chunks.active_idx);
    e_load_chunk.send(LoadChunkEvent { idx: chunks.active_idx, is_active: true });

    let (x, y, z) = chunk_xyz(chunks.active_idx);

    let mut neighbors = vec![];

    if y < MAP_SIZE.1 - 1 {
        let north_idx = chunk_idx(x, y + 1, z);
        neighbors.push(north_idx);

        if x < MAP_SIZE.0 - 1 {
            let north_east_idx = chunk_idx(x + 1, y + 1, z);
            neighbors.push(north_east_idx);
        }

        if x > 0 {
            let north_west_idx = chunk_idx(x - 1, y + 1, z);
            neighbors.push(north_west_idx);
        }
    }

    if y > 0 {
        let south_idx = chunk_idx(x, y - 1, z);
        neighbors.push(south_idx);

        if x < MAP_SIZE.0 - 1 {
            let south_east_idx = chunk_idx(x + 1, y - 1, z);
            neighbors.push(south_east_idx);
        }

        if x > 0 {
            let south_west_idx = chunk_idx(x - 1, y - 1, z);
            neighbors.push(south_west_idx);
        }
    }

    if x < MAP_SIZE.0 - 1 {
        let east_idx = chunk_idx(x + 1, y, z);
        neighbors.push(east_idx);
    }

    if x > 0 {
        let west_idx = chunk_idx(x - 1, y, z);
        neighbors.push(west_idx);
    }

    for idx in neighbors.iter() {
        e_load_chunk.send(LoadChunkEvent::new(*idx));
    }

    for chunk in q_chunks.iter() {
        if !neighbors.contains(&chunk.idx()) && chunk.idx() != chunks.active_idx {
            e_unload_chunk.send(UnloadChunkEvent { idx: chunk.idx() });
        }
    };

}

fn on_unload_chunk(mut e_unload_chunk: EventReader<UnloadChunkEvent>, mut cmds: Commands, q_chunks: Query<(Entity, &Chunk)>) {
    for e in e_unload_chunk.read() {
        let Some((chunk_e, _)) = q_chunks.iter().find(|(_, c)| c.idx() == e.idx) else {
            continue;
        };
        
        info!("unload chunk! {}", e.idx);
        cmds.entity(chunk_e).despawn_recursive();
    }
}

fn on_load_chunk(mut e_load_chunk: EventReader<LoadChunkEvent>, mut cmds: Commands, mut q_chunks: Query<&mut Chunk>) {
    for e in e_load_chunk.read() {
        let status = if e.is_active { ChunkStatus::Active } else { ChunkStatus::Dormant };
        // if this chunk already exists, make sure the status matches expected
        if let Some(mut existing) = q_chunks.iter_mut().find(|x| x.idx() == e.idx) {
            if existing.is_active != e.is_active {
                info!("change chunk status {} -> {}", e.idx, e.is_active);
                existing.is_active = e.is_active;

                for tile in existing.tiles.iter() {
                    cmds.entity(*tile).insert(status);
                }
            }

            continue;
        }

        info!("spawn chunk {}", e.idx);
        let chunk_e = cmds.spawn((
            Name::new(format!("chunk-{}", e.idx)),
            Transform::default(),
            Visibility::Visible,
        )).id();

        let tiles = Grid::init_fill(CHUNK_SIZE.0, CHUNK_SIZE.1, |x, y| {
            let wpos = chunk_local_to_world(e.idx, x, y);

            let tile_id = cmds.spawn((
                Glyph {
                    idx: 0,
                    fg: Color::srgb_u8(35, 37, 37),
                    bg: Color::srgb_u8(0, 0, 0)
                },
                Position::new(wpos.0, wpos.1, wpos.2, Z_LAYER_GROUND),
                status,
            )).set_parent(chunk_e).id();

            tile_id
        });

        let mut chunk = Chunk::new(e.idx, tiles);
        chunk.is_active = e.is_active;

        cmds.entity(chunk_e).insert(chunk);
    }
}
