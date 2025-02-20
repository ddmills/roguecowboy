use bevy::prelude::*;

use crate::{
    common::{astar, AStarSettings, Distance, Grid, Perlin, Rand},
    glyph::{Glyph, Position},
    player::{Player, PlayerMovedEvent},
    projection::{chunk_local_to_world, world_to_chunk_idx, CHUNK_SIZE, Z_LAYER_GROUND},
    save::{save_chunk, try_load_chunk},
};

use super::{Chunk, ChunkSave, ChunkStatus, Chunks, Map, Terrain};

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

pub fn on_load_chunk(
    mut e_load_chunk: EventReader<LoadChunkEvent>,
    mut e_spawn_chunk: EventWriter<SpawnChunkEvent>,
    map: Res<Map>,
) {
    for LoadChunkEvent(chunk_idx) in e_load_chunk.read() {
        info!("load chunk! {}", chunk_idx);

        if let Some(save_data) = try_load_chunk(*chunk_idx) {
            e_spawn_chunk.send(SpawnChunkEvent { data: save_data });
            continue;
        };

        let nz = Perlin::new(*chunk_idx as u32, 0.01, 6, 2.12);

        // todo: snapshots

        let constraints = map.get_chunk_constraints(*chunk_idx);

        let mut r = Rand::seed(*chunk_idx as u64);
        let terrains = vec![Terrain::Grass];
        let mut terrain = Grid::init_fill(CHUNK_SIZE.0, CHUNK_SIZE.1, |x, y| r.pick(&terrains));

        let mut rivers = vec![];
        let mut footpaths = vec![];

        for (x, s) in constraints.south.iter().enumerate() {
            if *s == 1 {
                rivers.push((x, 0));
            }
            if *s == 2 {
                footpaths.push((x, 0));
            }
        }

        for (x, n) in constraints.north.iter().enumerate() {
            if *n == 1 {
                rivers.push((x, CHUNK_SIZE.1 - 1));
            }
            if *n == 2 {
                footpaths.push((x, CHUNK_SIZE.1 - 1));
            }
        }

        for (y, w) in constraints.west.iter().enumerate() {
            if *w == 1 {
                rivers.push((0, y));
            }
            if *w == 2 {
                footpaths.push((0, y));
            }
        }

        for (y, e) in constraints.east.iter().enumerate() {
            if *e == 1 {
                rivers.push((CHUNK_SIZE.0 - 1, y));
            }
            if *e == 2 {
                footpaths.push((CHUNK_SIZE.0 - 1, y));
            }
        }

        if footpaths.len() == 1 {
            footpaths.push((CHUNK_SIZE.0 / 2, CHUNK_SIZE.1 / 2));
        }

        if rivers.len() == 1 {
            rivers.push((CHUNK_SIZE.0 / 2, CHUNK_SIZE.1 / 2));
        }

        // every river should attempt to connect to every other river
        for (p1_idx, p1) in rivers.iter().enumerate() {
            (p1_idx..rivers.len()).for_each(|p2_idx| {
                let p2 = rivers[p2_idx];

                if p1_idx == p2_idx {
                    return;
                }

                info!("start {},{}", p1.0, p1.1);
                info!("goal {},{}", p2.0, p2.1);

                // find path between p1/p2, and set terrain!
                let result = astar(AStarSettings {
                    start: [
                        p1.0,
                        p1.1,
                        0,
                    ],
                    is_goal: |p| {
                        p[0] == p2.0 && p[1] == p2.1
                    },
                    cost: |a, b| {
                        let dist = Distance::diagonal([a[0] as i32, a[1] as i32, a[2] as i32], [b[0] as i32, b[1] as i32, b[2] as i32]);
                        let t = terrain.get(b[0], b[1]).unwrap();

                        dist * match t {
                            Terrain::Grass => 15.,
                            Terrain::Dirt => 15.,
                            Terrain::River => 1.,
                            Terrain::Footpath => 10.,
                        }
                    },
                    heuristic: |v| {
                        Distance::diagonal([v[0] as i32, v[1] as i32, v[2] as i32], [p2.0 as i32, p2.1 as i32, 0])
                    },
                    neighbors: |v| {
                        let mut n = vec![];

                        let [x, y, _] = v;

                        if x > 0 {
                            n.push([x - 1, y, 0]);
                        }

                        if x < CHUNK_SIZE.0 - 1 {
                            n.push([x + 1, y, 0]);
                        }

                        if y > 0 {
                            n.push([x, y - 1, 0]);
                        }

                        if y < CHUNK_SIZE.1 - 1 {
                            n.push([x, y + 1, 0]);
                        }

                        n
                    },
                    max_depth: 1000
                });

                if result.is_success {
                    // info!("path {}", result.path.iter().map(|[a, b, c]| format!("{},{}", a, b)).collect::<Vec<_>>().join(" "));
                    for [x, y, _] in result.path {
                        if x < CHUNK_SIZE.0 && y < CHUNK_SIZE.1 {
                            terrain.set(x, y, Terrain::River);
                        }
                    }
                } else {
                    info!("Failure!");
                }
            });
        }

        // every footpath should attempt to connect to every other footpath
        for (p1_idx, p1) in footpaths.iter().enumerate() {
            (p1_idx..footpaths.len()).for_each(|p2_idx| {
                let p2 = footpaths[p2_idx];

                if p1_idx == p2_idx {
                    return;
                }

                info!("start {},{}", p1.0, p1.1);
                info!("goal {},{}", p2.0, p2.1);

                // find path between p1/p2, and set terrain!
                let result = astar(AStarSettings {
                    start: [
                        p1.0,
                        p1.1,
                        0,
                    ],
                    is_goal: |p| {
                        p[0] == p2.0 && p[1] == p2.1
                    },
                    cost: |a, b| {
                        let dist = Distance::diagonal([a[0] as i32, a[1] as i32, a[2] as i32], [b[0] as i32, b[1] as i32, b[2] as i32]);
                        let t = terrain.get(b[0], b[1]).unwrap();

                        dist * match t {
                            Terrain::Grass => 5.,
                            Terrain::Dirt => 5.,
                            Terrain::River => 100.,
                            Terrain::Footpath => 1.,
                        }
                    },
                    heuristic: |v| {
                        Distance::diagonal([v[0] as i32, v[1] as i32, v[2] as i32], [p2.0 as i32, p2.1 as i32, 0])
                    },
                    neighbors: |v| {
                        let mut n = vec![];

                        let [x, y, _] = v;

                        if x > 0 {
                            n.push([x - 1, y, 0]);

                            if y > 0 {
                                n.push([x - 1, y - 1, 0]);
                            }

                            if y < CHUNK_SIZE.1 - 1 {
                                n.push([x - 1, y + 1, 0]);
                            }
                        }

                        if x < CHUNK_SIZE.0 - 1 {
                            n.push([x + 1, y, 0]);

                            if y > 0 {
                                n.push([x + 1, y - 1, 0]);
                            }

                            if y < CHUNK_SIZE.1 - 1 {
                                n.push([x + 1, y + 1, 0]);
                            }
                        }

                        if y > 0 {
                            n.push([x, y - 1, 0]);
                        }

                        if y < CHUNK_SIZE.1 - 1 {
                            n.push([x, y + 1, 0]);
                        }

                        n
                    },
                    max_depth: 1000
                });

                if result.is_success {
                    // info!("path {}", result.path.iter().map(|[a, b, c]| format!("{},{}", a, b)).collect::<Vec<_>>().join(" "));
                    for [x, y, _] in result.path {
                        if x < CHUNK_SIZE.0 && y < CHUNK_SIZE.1 {
                            terrain.set(x, y, Terrain::Footpath);
                        }
                    }
                } else {
                    info!("Failure!");
                }
            });
        }

        let data = ChunkSave {
            idx: *chunk_idx,
            terrain,
        };

        e_spawn_chunk.send(SpawnChunkEvent { data });
    }
}

pub fn on_unload_chunk(
    mut e_unload_chunk: EventReader<UnloadChunkEvent>,
    mut cmds: Commands,
    q_chunks: Query<(Entity, &Chunk)>,
) {
    for UnloadChunkEvent(chunk_idx) in e_unload_chunk.read() {
        info!("unload chunk! {}", chunk_idx);

        let Some((chunk_e, chunk)) = q_chunks.iter().find(|(_, c)| c.idx() == *chunk_idx) else {
            continue;
        };

        save_chunk(&chunk.to_save());

        cmds.entity(chunk_e).despawn_recursive();
    }
}

pub fn on_spawn_chunk(mut e_spawn_chunk: EventReader<SpawnChunkEvent>, mut cmds: Commands) {
    for e in e_spawn_chunk.read() {
        info!("spawn chunk! {}", e.data.idx);
        let chunk_e = cmds
            .spawn((
                Name::new(format!("chunk-{}", e.data.idx)),
                Transform::default(),
                Visibility::Hidden,
                ChunkStatus::Dormant,
            ))
            .id();

        let mut tiles = vec![];

        for x in 0..CHUNK_SIZE.0 {
            for y in 0..CHUNK_SIZE.1 {
                let terrain = e.data.terrain.get(x, y).unwrap();
                let wpos = chunk_local_to_world(e.data.idx, x, y);

                let tile_id = cmds
                    .spawn((
                        Glyph {
                            idx: terrain.sprite_idx(),
                            fg: Color::srgb_u8(35, 37, 37),
                            bg: Color::srgb_u8(0, 0, 0),
                        },
                        Position::new(wpos.0, wpos.1, wpos.2, Z_LAYER_GROUND),
                        ChunkStatus::Dormant,
                    ))
                    .set_parent(chunk_e)
                    .id();

                tiles.push(tile_id);
            }
        }

        let tile_grid = Grid::init_from_vec(CHUNK_SIZE.0, CHUNK_SIZE.1, tiles);
        let chunk = Chunk::new(e.data.idx, e.data.terrain.clone(), tile_grid);

        cmds.entity(chunk_e).insert(chunk);
    }
}

pub fn on_set_chunk_status(
    mut e_set_chunk_status: EventReader<SetChunkStatusEvent>,
    mut cmds: Commands,
    q_chunks: Query<(Entity, &Chunk)>,
    q_player: Query<&Position, With<Player>>,
) {
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
    mut chunks: ResMut<Chunks>,
) {
    for e in e_player_moved.read() {
        // let player = q_player.single();
        let player_chunk_idx = world_to_chunk_idx(e.x, e.y, e.z);

        if !chunks.active.contains(&player_chunk_idx) {
            chunks.active = vec![player_chunk_idx];
        }
    }
}
