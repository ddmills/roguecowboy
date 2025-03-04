use bevy::prelude::*;
use camera::CameraPlugin;
use player::PlayerPlugin;
use rendering::{setup_tileset, BevyColorable, GlyphPlugin, GlyphTextPlugin, Palette, TilesetTextures};
use ui::{UiPlugin, ViewportPlugin};
use world::{MapPlugin, ZoneSnapshotPlugin};

mod camera;
mod common;
mod player;
mod projection;
mod save;
mod world;
mod rendering;
mod ui;

#[derive(Default, States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Snapshot,
}

pub fn go_to_state(state: GameState) -> impl Fn(ResMut<NextState<GameState>>) {
    move |mut next| {
        next.set(state);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(CameraPlugin)
        .add_plugins(ZoneSnapshotPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(GlyphPlugin)
        .add_plugins(GlyphTextPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(ViewportPlugin)
        .init_state::<GameState>()
        .insert_resource(ClearColor(0x000000_u32.to_bevy_color()))
        .init_resource::<TilesetTextures>()
        .add_systems(
            OnEnter(GameState::Loading),
            (setup_tileset, go_to_state(GameState::Playing)).chain(),
        )
        .run();
}
