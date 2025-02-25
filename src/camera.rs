use bevy::{math::vec3, prelude::*, window::PrimaryWindow};

use crate::{
    player::Player, projection::{
        world_to_zone_idx, zone_transform_center, MAP_SIZE_F32, TEXEL_SIZE_F32, TILE_SIZE_F32, ZONE_SIZE_F32
    }, rendering::Position, GameState
};

pub struct CameraPlugin;
#[derive(Component)]
pub struct MainCamera;

#[derive(Resource, Default)]
pub struct CursorPosition {
    pub x: usize,
    pub y: usize,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPosition>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                camera_follow_player.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, close_on_esc)
            .add_systems(Update, on_mouse_move);
    }
}

fn setup_camera(mut cmds: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = 1. / TEXEL_SIZE_F32;
    cmds.spawn((Camera2d, MainCamera, projection, Msaa::Off));
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

    let zone_idx = player.zone_idx();
    let center_of_zone = zone_transform_center(zone_idx);
    let target = vec3(center_of_zone.0, center_of_zone.1, 0.);
    let lerped = camera.translation.lerp(target, a * speed);

    camera.translation = lerped;
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

pub fn on_mouse_move(
    mut cursor: ResMut<CursorPosition>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Ok(window) = q_windows.get_single() else {
        return;
    };

    let Some(viewport_position) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = q_camera.get_single() else {
        return;
    };

    let Ok(world_2d) = camera.viewport_to_world_2d(camera_transform, viewport_position) else {
        return;
    };

    cursor.x = ((world_2d.x / TILE_SIZE_F32.0) + 0.5).floor().clamp(0., ZONE_SIZE_F32.0 * MAP_SIZE_F32.0) as usize;
    cursor.y = ((world_2d.y / TILE_SIZE_F32.1) + 0.5).floor().clamp(0., ZONE_SIZE_F32.1 * MAP_SIZE_F32.1) as usize;
}
