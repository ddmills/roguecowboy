use bevy::prelude::*;

use crate::{
    camera::Layer, common::Grid, player::PlayerMovedEvent, projection::{world_to_zone_idx, zone_local_to_world, ZONE_SIZE, Z_LAYER_GROUND}, rendering::{Glyph, Position}, save::{save_zone, try_load_zone}, world::{SimpleZoneBuilder, ZoneBuilder, ENABLE_ZONE_SNAPSHOTS}
};

use super::{Map, Zone, ZoneData, ZoneSnapshotsEvent, ZoneStatus, Zones};

#[derive(Event)]
pub struct LoadZoneEvent(pub usize);

#[derive(Event)]
pub struct UnloadZoneEvent(pub usize);

#[derive(Event)]
pub struct SetZoneStatusEvent {
    pub idx: usize,
    pub status: ZoneStatus,
}

#[derive(Event)]
pub struct SpawnZoneEvent {
    pub data: ZoneData,
}

pub fn on_load_zone(
    mut e_load_zone: EventReader<LoadZoneEvent>,
    mut e_spawn_zone: EventWriter<SpawnZoneEvent>,
    mut e_zone_snapshots: EventWriter<ZoneSnapshotsEvent>,
    map: Res<Map>,
) {
    for LoadZoneEvent(zone_idx) in e_load_zone.read() {
        info!("load zone! {}", zone_idx);

        if let Some(save_data) = try_load_zone(*zone_idx) {
            e_spawn_zone.send(SpawnZoneEvent { data: save_data });
            continue;
        };

        let mut builder = SimpleZoneBuilder::default();
        let constraints = map.get_zone_constraints(*zone_idx);

        let data = builder.build(constraints);

        if ENABLE_ZONE_SNAPSHOTS {
            e_zone_snapshots.send(ZoneSnapshotsEvent {
                idx: *zone_idx,
                snapshots: builder.get_snapshots(),
            });
        }

        e_spawn_zone.send(SpawnZoneEvent { data });
    }
}

pub fn on_unload_zone(
    mut e_unload_zone: EventReader<UnloadZoneEvent>,
    mut cmds: Commands,
    q_zones: Query<(Entity, &Zone)>,
) {
    for UnloadZoneEvent(zone_idx) in e_unload_zone.read() {
        info!("unload zone! {}", zone_idx);

        let Some((zone_e, zone)) = q_zones.iter().find(|(_, c)| c.idx() == *zone_idx) else {
            continue;
        };

        save_zone(&zone.to_save());

        cmds.entity(zone_e).despawn_recursive();
    }
}

pub fn on_spawn_zone(mut e_spawn_zone: EventReader<SpawnZoneEvent>, mut cmds: Commands) {
    for e in e_spawn_zone.read() {
        info!("spawn zone! {}", e.data.idx);
        let zone_e = cmds
            .spawn((
                Name::new(format!("zone-{}", e.data.idx)),
                Transform::default(),
                Visibility::Hidden,
                ZoneStatus::Dormant,
            ))
            .id();

        let mut tiles = vec![];

        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                let terrain = e.data.terrain.get(x, y).unwrap();
                let wpos = zone_local_to_world(e.data.idx, x, y);
                let (bg, fg) = terrain.colors();

                let tile_id = cmds
                    .spawn((
                        Glyph {
                            tile: Some(terrain.tile()),
                            bg,
                            fg1: fg,
                            fg2: fg,
                            outline: None,
                            is_shrouded: true,
                        },
                        Position::new(wpos.0, wpos.1, wpos.2, Layer::Background),
                        ZoneStatus::Dormant,
                    ))
                    .set_parent(zone_e)
                    .id();

                tiles.push(tile_id);
            }
        }

        let tile_grid = Grid::init_from_vec(ZONE_SIZE.0, ZONE_SIZE.1, tiles);
        let zone = Zone::new(e.data.idx, e.data.terrain.clone(), tile_grid);

        cmds.entity(zone_e).insert(zone);
    }
}

pub fn on_set_zone_status(
    mut e_set_zone_status: EventReader<SetZoneStatusEvent>,
    mut cmds: Commands,
    q_zones: Query<(Entity, &Zone)>,
) {
    for e in e_set_zone_status.read() {
        let Some((zone_e, zone)) = q_zones.iter().find(|(_, c)| c.idx() == e.idx) else {
            continue;
        };

        cmds.entity(zone_e).insert(e.status);

        for tile in zone.tiles.iter() {
            cmds.entity(*tile).insert(e.status);
        }
    }
}

// check when player moves to a different zone and set it as active
pub fn on_player_move(
    mut e_player_moved: EventReader<PlayerMovedEvent>,
    // q_player: Query<&Position, With<Player>>,
    mut zones: ResMut<Zones>,
) {
    for e in e_player_moved.read() {
        // let player = q_player.single();
        let player_zone_idx = world_to_zone_idx(e.x, e.y, e.z);

        zones.player = player_zone_idx;

        if !zones.active.contains(&player_zone_idx) {
            zones.active = vec![player_zone_idx];
        }
    }
}
