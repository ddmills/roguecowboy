use bevy::{math::{vec2, vec3}, prelude::*, render::{render_resource::Extent3d, view::{self, RenderLayers}}};

use crate::{camera::{setup_cameras, CameraPosition, GameRenderTarget, Layer}, projection::TILE_SIZE, rendering::{BevyColorable, Palette}};

pub struct ViewportPlugin;

impl Plugin for ViewportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewportDim>()
            .add_systems(Startup, spawn_game_panel.after(setup_cameras))
            .add_systems(Startup, spawn_left_panel)
            .add_systems(Startup, spawn_console_panel)
            .add_systems(Update, (
                update_game_panel,
                update_left_panel,
                update_console_panel,
            ).run_if(resource_changed::<ViewportDim>).run_if(resource_changed::<CameraPosition>));
    }
}

#[derive(Default)]
pub struct Box {
    pub bottom: usize,
    pub left: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Resource, Default)]
pub struct ViewportDim {
    pub left_panel: Box,
    pub game: Box,
    pub console: Box,
    pub window_size_tiles: (usize, usize),
}


#[derive(Component)]
pub struct PanelGame;

#[derive(Component)]
pub struct PanelLeft;

#[derive(Component)]
pub struct PanelConsole;

pub fn spawn_game_panel(
    mut cmds: Commands,
    render_target: Res<GameRenderTarget>,
) {
    let z = Layer::TargetTexture.z();

    cmds.spawn((
        Sprite::from_image(render_target.0.clone_weak()),
        PanelGame,
        Transform::from_translation(vec3(0., 0., z)),
        RenderLayers::layer(Layer::TargetTexture as usize),
    ));
}

pub fn update_game_panel(
    render_target: Res<GameRenderTarget>,
    mut images: ResMut<Assets<Image>>,
    viewport: Res<ViewportDim>,
    cam_pos: Res<CameraPosition>,
    mut q_panel_game_transform: Single<&mut Transform, With<PanelGame>>,
) {
    let Some(render_target) = images.get_mut(&render_target.0) else {
        return;
    };

    if viewport.game.width == 0 || viewport.game.height == 0 {
        return;
    }

    let extent = Extent3d {
        width: (viewport.game.width * TILE_SIZE.0) as u32,
        height: (viewport.game.height * TILE_SIZE.1) as u32,
        ..default()
    };

    render_target.resize(extent);

    let offset = vec2(
        (viewport.game.left * TILE_SIZE.0) as f32,
        (viewport.game.bottom * TILE_SIZE.1) as f32
    );

    let pos = vec3(
        cam_pos.bottom_left.x + offset.x + (extent.width / 2) as f32,
        cam_pos.bottom_left.y + offset.y + (extent.height / 2) as f32,
        Layer::TargetTexture.z() + 1.
    ).floor();

    q_panel_game_transform.translation = pos;
}


pub fn spawn_left_panel(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let z = Layer::TargetTexture.z();

    cmds.spawn((
        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial2d(materials.add(Palette::Green.to_bevy_color())),
        PanelLeft,
        Transform::from_translation(vec3(0., 0., z)),
        RenderLayers::layer(Layer::TargetTexture as usize),
    ));
}

pub fn update_left_panel(
    viewport: Res<ViewportDim>,
    cam_pos: Res<CameraPosition>,
    mut q_panel_left_transform: Single<&mut Transform, With<PanelLeft>>,
) {
    if viewport.game.width == 0 || viewport.game.height == 0 {
        return;
    }

    info!("RESIZE ME - {},{}", viewport.window_size_tiles.0, viewport.window_size_tiles.1);

    let scale = vec3(
        (viewport.left_panel.width * TILE_SIZE.0) as f32,
        (viewport.left_panel.height * TILE_SIZE.1) as f32,
        1.
    );

    let offset = vec2(
        (viewport.left_panel.left * TILE_SIZE.0) as f32,
        (viewport.left_panel.bottom * TILE_SIZE.1) as f32
    );

    let pos = vec3(
        (cam_pos.bottom_left.x + offset.x) as f32 + (scale.x / 2.),
        (cam_pos.bottom_left.y + offset.y) as f32 + (scale.y / 2.),
        Layer::TargetTexture.z() + 1.
    ).floor();

    q_panel_left_transform.scale = scale;
    q_panel_left_transform.translation = pos;
}


pub fn spawn_console_panel(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let z = Layer::TargetTexture.z();

    cmds.spawn((
        Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial2d(materials.add(Palette::Brown.to_bevy_color())),
        PanelConsole,
        Transform::from_translation(vec3(0., 0., z)),
        RenderLayers::layer(Layer::TargetTexture as usize),
    ));
}

pub fn update_console_panel(
    viewport: Res<ViewportDim>,
    cam_pos: Res<CameraPosition>,
    mut q_panel_left_transform: Single<&mut Transform, With<PanelConsole>>,
) {
    if viewport.game.width == 0 || viewport.game.height == 0 {
        return;
    }

    let scale = vec3(
        (viewport.console.width * TILE_SIZE.0) as f32,
        (viewport.console.height * TILE_SIZE.1) as f32,
        1.
    );

    let offset = vec2(
        (viewport.console.left * TILE_SIZE.0) as f32,
        (viewport.console.bottom * TILE_SIZE.1) as f32
    );

    let pos = vec3(
        (cam_pos.bottom_left.x + offset.x) as f32 + (scale.x / 2.),
        (cam_pos.bottom_left.y + offset.y) as f32 + (scale.y / 2.),
        Layer::TargetTexture.z() + 1.
    ).floor();

    q_panel_left_transform.scale = scale;
    q_panel_left_transform.translation = pos;
}
