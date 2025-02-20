use bevy::{math::vec3, prelude::*};

use crate::{
    glyph::Position, player::Player, projection::{
        chunk_xyz, world_to_chunk_idx, CHUNK_SIZE, CHUNK_SIZE_F32, TEXEL_SIZE_F32, TILE_SIZE, TILE_SIZE_F32
    }, GameState
};

pub struct CameraPlugin;
#[derive(Component)]
pub struct MainCamera;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                camera_follow_player.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, close_on_esc);
    }
}

fn setup_camera(mut cmds: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 1. / TEXEL_SIZE_F32;
    cmds.spawn((Camera2d, MainCamera, projection));
}

pub fn camera_follow_player(
    mut q_camera: Query<&mut Transform, With<MainCamera>>,
    q_player: Query<&Position, With<Player>>,
    fixed_time: Res<Time<Fixed>>,
) {
    let mut camera = q_camera.single_mut();
    let player = q_player.single();
    let a = fixed_time.overstep_fraction();
    let speed = 0.1;

    let chunk_idx = world_to_chunk_idx(player.x, player.y, player.z);
    let chunk_pos = chunk_xyz(chunk_idx);
    let center_of_chunk = (
        (chunk_pos.0 * CHUNK_SIZE.0 * TILE_SIZE.0) as f32
            + ((CHUNK_SIZE_F32.0 * TILE_SIZE_F32.0) / 2.)
            - (TILE_SIZE_F32.0 / 2.),
        (chunk_pos.1 * CHUNK_SIZE.1 * TILE_SIZE.1) as f32
            + ((CHUNK_SIZE_F32.1 * TILE_SIZE_F32.1) / 2.)
            - (TILE_SIZE_F32.1 / 2.),
    );

    let new_pos = vec3(center_of_chunk.0 as f32, center_of_chunk.1 as f32, 0.);
    let target = camera.translation.lerp(new_pos, a * speed);

    camera.translation = target;
}

pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}
