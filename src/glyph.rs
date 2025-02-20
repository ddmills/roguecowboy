use bevy::{math::vec2, prelude::*};

use crate::{
    GameState,
    projection::{TILE_SIZE, world_to_px},
    world::ZoneStatus,
};

pub struct GlyphPlugin;

impl Plugin for GlyphPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_glyph_sprites, update_positions, on_status_change)
        );
    }
}

#[derive(Component, Default)]
#[require(Sprite)]
pub struct Glyph {
    pub idx: usize,
    pub fg: Color,
}

#[derive(Component)]
#[require(Transform)]
pub struct Position {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub layer: usize,
}

impl Position {
    pub fn new(x: usize, y: usize, z: usize, layer: usize) -> Self {
        Self { x, y, z, layer }
    }
}

pub fn tile_translation(x: usize, y: usize) -> Vec2 {
    let px = world_to_px(x, y);
    vec2(px.0 as f32, px.1 as f32)
}

// update any sprites that have glyph changed
pub fn update_glyph_sprites(
    mut q_changed: Query<(&Glyph, &mut Sprite), Changed<Glyph>>,
    tileset: Res<Tileset>,
) {
    for (glyph, mut sprite) in q_changed.iter_mut() {
        sprite.image = tileset.texture.clone_weak();
        sprite.texture_atlas = Some(TextureAtlas {
            layout: tileset.layout.clone_weak(),
            index: glyph.idx,
        });
        sprite.color = glyph.fg;
    }
}

pub fn update_positions(mut q_changed: Query<(&Position, &mut Transform), Changed<Position>>) {
    for (position, mut transform) in q_changed.iter_mut() {
        let z = (10 * position.z + (10 - position.layer)) as f32;
        let target = tile_translation(position.x, position.y).extend(-z);
        transform.translation = target;
    }
}

#[derive(Resource, Default)]
pub struct Tileset {
    layout: Handle<TextureAtlasLayout>,
    texture: Handle<Image>,
}

pub fn setup_tileset(
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut tileset: ResMut<Tileset>,
) {
    tileset.texture = asset_server.load("tileset.png");
    tileset.layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(TILE_SIZE.0 as u32, TILE_SIZE.1 as u32),
        8,
        8,
        None,
        None,
    ));
}

pub fn on_status_change(mut q_changed: Query<(&mut Glyph, &ZoneStatus), Changed<ZoneStatus>>) {
    for (mut glyph, status) in q_changed.iter_mut() {
        if *status == ZoneStatus::Active {
            glyph.fg = Color::srgb_u8(255, 255, 255);
        } else {
            glyph.fg = Color::srgb_u8(92, 97, 97);
        }
    }
}
