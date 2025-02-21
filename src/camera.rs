use bevy::{math::vec3, prelude::*};

use crate::{
    glyph::Position, player::Player, projection::{
        world_to_zone_idx, zone_transform_center, zone_xyz, TEXEL_SIZE_F32, TILE_SIZE, TILE_SIZE_F32, ZONE_SIZE, ZONE_SIZE_F32
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

    let zone_idx = world_to_zone_idx(player.x, player.y, player.z);
    let center_of_zone = zone_transform_center(zone_idx);
    let new_pos = vec3(center_of_zone.0, center_of_zone.1, 0.);
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
