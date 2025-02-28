use bevy::{asset::RenderAssetUsages, math::{vec2, vec3}, prelude::*, render::{camera::ScalingMode, mesh, render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages}, view::{self, RenderLayers}}, sprite::Material2d, window::{PrimaryWindow, WindowResized}};

use crate::{
    player::Player, projection::{
        zone_transform_center, MAP_SIZE_F32, TEXEL_SIZE, TEXEL_SIZE_F32, TILE_SIZE, TILE_SIZE_F32, ZONE_SIZE_F32
    }, rendering::{BevyColorable, Palette, Position}, ui::{PanelGame, PanelLeft, ViewportDim}, GameState
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

#[derive(Default, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Layer {
    Background = 1,
    #[default]
    Actors = 2,
    Fx = 4,
    UiLayout = 10,
    Ui = 11,
    TargetTexture = 50,
}

impl Layer {
    pub fn get_screen_space() -> Vec<usize>
    {
        vec![
            Self::Ui as usize,
            Self::UiLayout as usize,
            Self::TargetTexture as usize,
        ]
    }

    pub fn get_world_space() -> Vec<usize>
    {
        vec![
            Self::Background as usize,
            Self::Actors as usize,
            Self::Fx as usize,
        ]
    }

    pub fn is_screen(&self) -> bool
    {
        match self {
            Layer::Background => false,
            Layer::Actors => false,
            Layer::Fx => false,
            Layer::UiLayout => true,
            Layer::Ui => true,
            Layer::TargetTexture => true,
        }
    }

    pub fn z(&self) -> f32
    {
        let z = match self {
            Layer::Background => 0.,
            Layer::Actors => 4.,
            Layer::Fx => 5.,
            Layer::UiLayout => 49.,
            Layer::Ui => 50.,
            Layer::TargetTexture => 44.,
        };

        -(100. - z)
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorPosition>()
            .init_resource::<GameRenderTarget>()
            .init_resource::<CameraPosition>()
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

#[derive(Resource, Default)]
pub struct GameRenderTarget(pub Handle<Image>);

#[derive(Resource, Default)]
pub struct CameraPosition {
    pub bottom_left: Vec2,
}

pub fn setup_cameras(
    mut cmds: Commands,
    mut images: ResMut<Assets<Image>>,
    mut render_target: ResMut<GameRenderTarget>,
) {
    let render_target_image = make_render_target(400, 200);
    let render_target_handle = images.add(render_target_image);
    let mut p = OrthographicProjection::default_2d();
    p.scale = 1. / TEXEL_SIZE_F32;

    // SCREEN SPACE CAMERA
    cmds.spawn((
        Camera2d,
        MainCamera,
        Msaa::Off,
        Projection::Orthographic(p),
        RenderLayers::from_layers(&Layer::get_screen_space()),
    ));

    // WORLD SPACE CAMERA
    cmds.spawn((
        Camera2d,
        Camera {
            clear_color: Palette::Black.to_bevy_color().into(),
            target: render_target_handle.clone().into(),
            ..default()
        },
        GameCamera,
        Msaa::Off,
        RenderLayers::from_layers(&Layer::get_world_space()),
    ));

    render_target.0 = render_target_handle;
}

fn on_resize_window(
    mut ev_window_resized: EventReader<WindowResized>,
    mut viewport: ResMut<ViewportDim>,
    mut cam_pos: ResMut<CameraPosition>,
    main_camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if ev_window_resized.is_empty() {
        return;
    }
    ev_window_resized.clear();

    let (cam, cam_transform) = main_camera.into_inner();

    let y = viewport.window_size_tiles.1 * TILE_SIZE.1 * TEXEL_SIZE;
    let viewport_pos = vec2(0.0, y as f32);

    if let Ok(world_2d) = cam.viewport_to_world_2d(cam_transform, viewport_pos) {
        cam_pos.bottom_left = world_2d;
    };


    let console_h = 6;
    let left_panel_w = 12;

    let size = window.size();

    info!("RESIZED {},{}", size.x, size.y);

    let screen_w_tiles = (size.x / (TILE_SIZE_F32.0 * TEXEL_SIZE_F32)) as usize;
    let screen_h_tiles = (size.y / (TILE_SIZE_F32.1 * TEXEL_SIZE_F32)) as usize;

    viewport.window_size_tiles = (screen_w_tiles, screen_h_tiles);

    viewport.left_panel.bottom = 0;
    viewport.left_panel.left = 0;
    viewport.left_panel.width = left_panel_w;
    viewport.left_panel.height = screen_h_tiles;

    viewport.console.bottom = 0;
    viewport.console.left = left_panel_w;
    viewport.console.width = screen_w_tiles - left_panel_w;
    viewport.console.height = console_h;

    viewport.game.left = left_panel_w;
    viewport.game.bottom = console_h;
    viewport.game.width = screen_w_tiles - left_panel_w;
    viewport.game.height = screen_h_tiles - console_h;
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
    window: Single<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
) {
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
