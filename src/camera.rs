use bevy::{asset::RenderAssetUsages, math::{vec2, vec3}, prelude::*, render::{camera::ScalingMode, mesh, render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages}, view::RenderLayers}, sprite::Material2d, window::{PrimaryWindow, WindowResized}};

use crate::{
    player::Player, projection::{
        zone_transform_center, MAP_SIZE_F32, TEXEL_SIZE, TEXEL_SIZE_F32, TILE_SIZE_F32, ZONE_SIZE_F32
    }, rendering::{BevyColorable, Palette, Position}, GameState
};

pub struct CameraPlugin;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct GameCamera;

#[derive(Resource, Default)]
pub struct CursorPosition {
    pub x: usize,
    pub y: usize,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPosition>()
            .add_systems(Startup, setup_cameras)
            .add_systems(
                Update,
                camera_follow_player.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, on_resize_window)
            .add_systems(Update, close_on_esc)
            .add_systems(Update, on_mouse_move);
    }
}

fn make_render_target(width: u32, height: u32) -> Image
{
    let size = Extent3d {
        width,
        height,
        ..default()
    };

    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default()
    );

    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    image
}

fn setup_cameras(
    mut cmds: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let render_target = make_render_target(400, 200);
    let render_target_handle = images.add(render_target);

    cmds.spawn((
        Camera2d,
        MainCamera,
        Msaa::Off,
        RenderLayers::layer(1),
    ));

    cmds.spawn((
        Camera2d,
        Camera {
            clear_color: Palette::Black.to_bevy_color().into(),
            target: render_target_handle.clone().into(),
            ..default()
        },
        GameCamera,
        Msaa::Off,
        RenderLayers::layer(0),
    ));

    cmds.spawn((
        Sprite::from_image(render_target_handle),
        RenderLayers::layer(1),
        Transform::from_scale(Vec3::splat(TEXEL_SIZE_F32)),
    ));
}

fn on_resize_window(
    mut camera: Single<&mut Camera, With<GameCamera>>,
    mut ev_window_resized: EventReader<WindowResized>,
    mut images: ResMut<Assets<Image>>,
) {
    for e in ev_window_resized.read() {
        let width = e.width.floor() as u32;
        let height = e.height.floor() as u32;
        let Some(old_render_target_handle) = camera.target.as_image() else {
            return;
        };

        let Some(render_target) = images.get_mut(old_render_target_handle) else {
            return;
        };

        let dest_size = (vec2(width as f32, height as f32) / (TEXEL_SIZE_F32 * 2.)).floor() * TEXEL_SIZE_F32;

        let extent = Extent3d {
            width: dest_size.x as u32,
            height: dest_size.y as u32,
            ..default()
        };

        render_target.resize(extent);

        info!("{}/{} -> {}/{}", e.width, e.height, dest_size.x, dest_size.y);
    }
}

pub fn camera_follow_player(
    mut q_camera: Query<&mut Transform, With<GameCamera>>,
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

    camera.translation = target.floor();
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
    q_camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
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
