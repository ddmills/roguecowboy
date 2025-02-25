use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    common::{Grid, Grid3d, Rand}, player::Player, projection::{zone_idx, zone_xyz, MAP_SIZE, ZONE_SIZE}, rendering::{Palette, Position, Tile}, GameState
};

use super::{
    ENABLE_ZONE_SNAPSHOTS, LoadZoneEvent, SetZoneStatusEvent, SpawnZoneEvent, UnloadZoneEvent,
    ZoneConstraints, ZoneData, on_load_zone, on_player_move, on_set_zone_status, on_spawn_zone,
    on_unload_zone,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<Map>()
            .init_resource::<Zones>()
            .add_event::<LoadZoneEvent>()
            .add_event::<UnloadZoneEvent>()
            .add_event::<SetZoneStatusEvent>()
            .add_event::<SpawnZoneEvent>()
            .add_systems(
                Update,
                (
                    on_player_move,
                    load_nearby_zones,
                    on_load_zone,
                    on_unload_zone,
                    on_spawn_zone,
                    on_set_zone_status,
                    zone_visibility,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Clone, Resource)]
pub struct Map {
    zones: Grid3d<OverworldZone>,
}

impl Default for Map {
    fn default() -> Self {
        let zones = Grid3d::init(MAP_SIZE.0, MAP_SIZE.1, MAP_SIZE.2, OverworldZone);
        Self { zones }
    }
}

pub struct ZoneContinuity {
    pub south: Vec<u8>,
    pub west: Vec<u8>,
}

impl Map {
    fn get_continuity(&self, x: usize, y: usize, z: usize) -> ZoneContinuity {
        if self.zones.is_oob(x, y, z) {
            return ZoneContinuity {
                south: vec![],
                west: vec![],
            };
        }

        let idx = zone_idx(x, y, z);
        let mut rand = Rand::seed(idx as u64);

        let mut south = [0; ZONE_SIZE.0];
        let mut west = [0; ZONE_SIZE.1];

        if y > 0 {
            // river
            if x % 3 == 0 {
                let r = rand.range_n(1, ZONE_SIZE.0 as i32 - 1) as usize;
                south[r] = 1;
            }

            // path
            if x % 4 == 0 {
                let r = rand.range_n(1, ZONE_SIZE.0 as i32 - 1) as usize;
                south[r] = 2;
            }
        }

        if x > 0 {
            // river
            if y % 2 == 0 {
                let r = rand.range_n(1, ZONE_SIZE.1 as i32 - 1) as usize;
                west[r] = 1;
            }

            // footpaths
            if y % 2 == 0 {
                let r = rand.range_n(1, ZONE_SIZE.1 as i32 - 1) as usize;
                west[r] = 2;
            }
        }

        ZoneContinuity {
            south: south.to_vec(),
            west: west.to_vec(),
        }
    }

    pub fn get_zone_constraints(&self, idx: usize) -> ZoneConstraints {
        let (x, y, z) = zone_xyz(idx);
        let own = self.get_continuity(x, y, z);

        let east = self.get_continuity(x + 1, y, z);
        let north = self.get_continuity(x, y + 1, z);

        ZoneConstraints {
            idx,
            north: north.south,
            west: own.west,
            south: own.south,
            east: east.west,
        }
    }
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum ZoneStatus {
    Active,
    Dormant,
}

#[derive(Resource, Default)]
pub struct Zones {
    pub active: Vec<usize>,
    pub player: usize,
}

#[derive(Clone, Component)]
pub struct Zone {
    terrain: Grid<Terrain>,
    pub tiles: Grid<Entity>,
    idx: usize,
}

impl Zone {
    pub fn new(idx: usize, terrain: Grid<Terrain>, tiles: Grid<Entity>) -> Self {
        Self {
            terrain,
            idx,
            tiles,
        }
    }

    #[inline]
    pub fn to_save(&self) -> ZoneData {
        ZoneData {
            idx: self.idx,
            terrain: self.terrain.clone(),
        }
    }

    #[inline]
    pub fn idx(&self) -> usize {
        self.idx
    }
}

#[derive(Clone, Default)]
pub struct OverworldZone;

#[repr(u8)]
#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum Terrain {
    #[default]
    Grass = 1,
    Dirt = 2,
    River = 3,
    Footpath = 4,
}

impl Terrain {
    pub fn sprite_ch(&self) -> char {
        match self {
            Terrain::Grass => '.',
            Terrain::Dirt => '.',
            Terrain::River => '~',
            Terrain::Footpath => '░',
        }
    }

    pub fn tile(&self) -> Tile {
        match self {
            Terrain::Grass => Tile::Grass,
            Terrain::Dirt => Tile::Dirt,
            Terrain::River => Tile::Water,
            Terrain::Footpath => Tile::Dirt,
        }
    }

    pub fn colors(&self) -> (Option<u32>, Option<u32>) {
        match self {
            Terrain::Grass => (None, Some(Palette::Green.into())),
            Terrain::Dirt => (None, Some(Palette::Brown.into())),
            Terrain::River => (Some(Palette::Blue.into()), Some(Palette::Cyan.into())),
            Terrain::Footpath => (None, Some(Palette::Brown.into())),
        }
    }
}

fn zone_visibility(
    mut cmds: Commands,
    q_player: Query<&Position, With<Player>>,
    q_zones: Query<(Entity, &Zone)>,
) {
    let Ok(player) = q_player.get_single() else {
        return;
    };

    for (zone_e, zone) in q_zones.iter() {
        let zone_pos = zone_xyz(zone.idx);

        // compare Z levels
        if zone_pos.2 != player.z.floor() as usize {
            cmds.entity(zone_e).insert(Visibility::Hidden);
        } else {
            cmds.entity(zone_e).insert(Visibility::Visible);
        }
    }
}

// determine which zones should
//  - be loaded
//  - be unloaded
//  - change status
fn load_nearby_zones(
    zones: Res<Zones>,
    mut e_load_zone: EventWriter<LoadZoneEvent>,
    mut e_unload_zone: EventWriter<UnloadZoneEvent>,
    mut e_set_zone_status: EventWriter<SetZoneStatusEvent>,
    q_zones: Query<(&Zone, &ZoneStatus)>,
) {
    let mut cur_dormant_zones = q_zones
        .iter()
        .filter_map(|(c, s)| match s {
            ZoneStatus::Active => None,
            ZoneStatus::Dormant => Some(c.idx()),
        })
        .collect::<Vec<_>>();

    let mut cur_active_zones = q_zones
        .iter()
        .filter_map(|(c, s)| match s {
            ZoneStatus::Active => Some(c.idx()),
            ZoneStatus::Dormant => None,
        })
        .collect::<Vec<_>>();

    let mut needed_zones = zones.active.clone();

    for idx in zones.active.iter() {
        let (x, y, z) = zone_xyz(*idx);

        if y < MAP_SIZE.1 - 1 {
            let north_idx = zone_idx(x, y + 1, z);
            needed_zones.push(north_idx);

            if x < MAP_SIZE.0 - 1 {
                let north_east_idx = zone_idx(x + 1, y + 1, z);
                needed_zones.push(north_east_idx);
            }

            if x > 0 {
                let north_west_idx = zone_idx(x - 1, y + 1, z);
                needed_zones.push(north_west_idx);
            }
        }

        if y > 0 {
            let south_idx = zone_idx(x, y - 1, z);
            needed_zones.push(south_idx);

            if x < MAP_SIZE.0 - 1 {
                let south_east_idx = zone_idx(x + 1, y - 1, z);
                needed_zones.push(south_east_idx);
            }

            if x > 0 {
                let south_west_idx = zone_idx(x - 1, y - 1, z);
                needed_zones.push(south_west_idx);
            }
        }

        if z > 0 {
            let above_idx = zone_idx(x, y, z - 1);
            needed_zones.push(above_idx);
        }

        if x < MAP_SIZE.0 - 1 {
            let east_idx = zone_idx(x + 1, y, z);
            needed_zones.push(east_idx);
        }

        if x > 0 {
            let west_idx = zone_idx(x - 1, y, z);
            needed_zones.push(west_idx);
        }

        if z < MAP_SIZE.2 - 1 {
            let below_idx = zone_idx(x, y, z + 1);
            needed_zones.push(below_idx);
        }
    }

    if ENABLE_ZONE_SNAPSHOTS {
        needed_zones = zones.active.clone();
    }

    let mut zones_to_load = vec![];
    let mut zones_to_dormant = vec![];
    let mut zones_to_active = vec![];

    needed_zones.sort();
    needed_zones.dedup();

    for idx in needed_zones.iter() {
        let is_active = zones.active.contains(idx);

        if let Some(cur_pos) = cur_active_zones.iter().position(|&i| i == *idx) {
            cur_active_zones.swap_remove(cur_pos);

            // zone is active, but needs to be dormant.
            if !is_active {
                zones_to_dormant.push(*idx);
            }
        } else if let Some(cur_pos) = cur_dormant_zones.iter().position(|&i| i == *idx) {
            cur_dormant_zones.swap_remove(cur_pos);

            // zone is dormant but needs to be active
            if is_active {
                zones_to_active.push(*idx);
            }
        } else {
            // zone is not dormant or active, but needed. We must load it
            zones_to_load.push(*idx);

            // also needs to be active
            if is_active {
                zones_to_active.push(*idx);
            }
        }
    }

    let zones_to_unload = [cur_active_zones, cur_dormant_zones].concat();

    if let Some(idx) = zones_to_load.first() {
        e_load_zone.send(LoadZoneEvent(*idx));
    }

    if let Some(idx) = zones_to_unload.first() {
        e_unload_zone.send(UnloadZoneEvent(*idx));
    }

    // for idx in zones_to_load.iter() {
    //     e_load_zone.send(LoadZoneEvent(*idx));
    // }

    // for idx in zones_to_unload.iter() {
    //     e_unload_zone.send(UnloadZoneEvent(*idx));
    // }

    for idx in zones_to_active.iter() {
        e_set_zone_status.send(SetZoneStatusEvent {
            idx: *idx,
            status: ZoneStatus::Active,
        });
    }

    for idx in zones_to_dormant.iter() {
        e_set_zone_status.send(SetZoneStatusEvent {
            idx: *idx,
            status: ZoneStatus::Dormant,
        });
    }
}
