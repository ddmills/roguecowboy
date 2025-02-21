use bevy::prelude::*;
use camera::CameraPlugin;
use glyph::{GlyphPlugin, Tileset, setup_tileset};
use player::PlayerPlugin;
use world::{MapPlugin, ZoneSnapshotPlugin};

mod camera;
mod common;
mod glyph;
mod player;
mod projection;
mod save;
mod world;

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
        .add_plugins(PlayerPlugin)
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::srgb_u8(19, 27, 37)))
        .init_resource::<Tileset>()
        .add_systems(
            OnEnter(GameState::Loading),
            (setup_tileset, go_to_state(GameState::Playing)).chain(),
        )
        .run();
}
