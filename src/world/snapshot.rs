use bevy::{math::vec3, prelude::*};

use crate::{
    camera::{CursorPosition, MainCamera}, common::Grid, projection::{
        world_to_zone_local, zone_local_to_world, zone_transform_center, ZONE_SIZE, Z_LAYER_SNAPSHOT
    }, rendering::{Glyph, Palette, Position, Tile}, GameState
};

use super::{ZoneSnapshot, Zones};

pub struct ZoneSnapshotPlugin;

impl Plugin for ZoneSnapshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ZoneSnapshotsEvent>()
            .add_event::<UpdateSnapshotTilesEvent>()
            .init_resource::<SnapshotMode>()
            .add_systems(OnEnter(GameState::Snapshot), enter_snapshot_mode)
            .add_systems(Update, on_zone_snapshot_event)
            .add_systems(
                Update,
                (snapshot_controls, on_update_snapshot_tiles, snapshot_cursor)
                    .chain()
                    .run_if(in_state(GameState::Snapshot)),
            );
    }
}

#[derive(Resource, Default)]
pub struct SnapshotMode {
    pub idx: usize,
    pub current_snap_idx: usize,
    pub snapshots: Vec<ZoneSnapshot>,
}

#[derive(Resource)]
pub struct SnapshotTiles {
    pub container: Entity,
    pub tiles: Grid<Entity>,
}

#[derive(Component)]
pub struct SnapshotTile;

#[derive(Event)]
pub struct ZoneSnapshotsEvent {
    pub idx: usize,
    pub snapshots: Vec<ZoneSnapshot>,
}

#[derive(Event)]
pub struct UpdateSnapshotTilesEvent {
    pub snap_idx: usize,
}

pub fn on_zone_snapshot_event(
    mut e_zone_snapshots: EventReader<ZoneSnapshotsEvent>,
    mut mode: ResMut<SnapshotMode>,
    zones: Res<Zones>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    for e in e_zone_snapshots.read() {
        if zones.player == e.idx {
            info!("got snapshots! {}", e.idx);
            mode.idx = e.idx;
            mode.snapshots = e.snapshots.clone();
            mode.current_snap_idx = 0;
            next_game_state.set(GameState::Snapshot);
        }
    }
}

pub fn enter_snapshot_mode(
    mut cmds: Commands,
    mode: Res<SnapshotMode>,
    mut q_camera: Query<&mut Transform, With<MainCamera>>,
    mut e_change_snapshot: EventWriter<UpdateSnapshotTilesEvent>,
) {
    let center_of_zone = zone_transform_center(mode.idx);
    let Ok(mut camera) = q_camera.get_single_mut() else {
        return;
    };

    camera.translation = vec3(center_of_zone.0, center_of_zone.1, 0.);

    let container = cmds
        .spawn((
            Name::new("snapshot"),
            Transform::default(),
            Visibility::Visible,
        ))
        .id();

    let mut tiles = vec![];

    for x in 0..ZONE_SIZE.0 {
        for y in 0..ZONE_SIZE.1 {
            let wpos = zone_local_to_world(mode.idx, x, y);

            let mut e = cmds.spawn((
                Glyph {
                    tile: Some(Tile::Blank),
                    fg1: None,
                    fg2: None,
                    bg: Some(Palette::Black.into()),
                    outline: None,
                    is_shrouded: false,
                },
                Position::new(wpos.0, wpos.1, wpos.2, Z_LAYER_SNAPSHOT),
                SnapshotTile,
            ));

            e.set_parent(container);
            tiles.push(e.id());
        }
    }

    cmds.insert_resource(SnapshotTiles {
        container,
        tiles: Grid::init_from_vec(ZONE_SIZE.0, ZONE_SIZE.1, tiles),
    });

    e_change_snapshot.send(UpdateSnapshotTilesEvent { snap_idx: 0 });
}

pub fn snapshot_cursor(cursor: Res<CursorPosition>, mode: Res<SnapshotMode>) {
    let Some(snapshot) = mode.snapshots.get(mode.current_snap_idx) else {
        return;
    };

    let (x, y) = world_to_zone_local(cursor.x, cursor.y);

    let Some(tile) = snapshot.data.get(x, y) else {
        return;
    };
}

pub fn snapshot_controls(
    mut cmds: Commands,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mode: ResMut<SnapshotMode>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut e_change_snapshot: EventWriter<UpdateSnapshotTilesEvent>,
    tiles: Res<SnapshotTiles>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        next_game_state.set(GameState::Playing);
        cmds.entity(tiles.container).despawn_recursive();
    }

    if keys.just_pressed(KeyCode::KeyW) && mode.current_snap_idx < mode.snapshots.len() - 1 {
        mode.current_snap_idx += 1;
        e_change_snapshot.send(UpdateSnapshotTilesEvent {
            snap_idx: mode.current_snap_idx,
        });
    }

    if keys.just_pressed(KeyCode::KeyS) && mode.current_snap_idx > 0 {
        mode.current_snap_idx -= 1;
        e_change_snapshot.send(UpdateSnapshotTilesEvent {
            snap_idx: mode.current_snap_idx,
        });
    }

    keys.reset_all();
}

pub fn on_update_snapshot_tiles(
    mut e_change_snapshot: EventReader<UpdateSnapshotTilesEvent>,
    mode: Res<SnapshotMode>,
    snapshot_tiles: Res<SnapshotTiles>,
    mut q_glyphs: Query<&mut Glyph, With<SnapshotTile>>,
) {
    for e in e_change_snapshot.read() {
        info!("redraw tiles! {}", e.snap_idx);

        let Some(snapshot) = mode.snapshots.get(e.snap_idx) else {
            continue;
        };

        for x in 0..ZONE_SIZE.0 {
            for y in 0..ZONE_SIZE.1 {
                let Some(snap_color) = snapshot.data.get(x, y) else {
                    continue;
                };

                let Some(entity) = snapshot_tiles.tiles.get(x, y) else {
                    continue;
                };

                let Ok(mut glyph) = q_glyphs.get_mut(*entity) else {
                    continue;
                };

                glyph.fg1 = Some(snap_color.to_color());
            }
        }
    }
}
