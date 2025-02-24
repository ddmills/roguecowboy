use bevy::{math::{vec2, vec3}, prelude::*, render::render_resource::AsBindGroup, sprite::{AlphaMode2d, Material2d, Material2dPlugin}};

use crate::{
    common::{cp437_idx, CP437_NBSP}, projection::{world_to_px, TILE_SIZE, TILE_SIZE_F32}, world::ZoneStatus
};

pub const CLEAR_COLOR: Color = Color::srgb(0.012, 0.059, 0.106);
pub const SHROUD_COLOR: Color = Color::srgb(0.227, 0.243, 0.247);
pub const TRANSPARENT: Color = Color::srgba(0.659, 0.294, 0.294, 0.);
pub const TEXT_COLOR: Color = Color::srgb(0.804, 0.867, 0.875);

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Grass = 3,
    Water = 34,
    Cowboy = 146,
    Dirt = 19,
    Blank = 255,
}

pub struct GlyphPlugin;

impl Plugin for GlyphPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(Material2dPlugin::<GlyphMaterial>::default())
            .add_systems(
            Update,
            (add_glyph_material, update_glyph_material, update_positions, on_status_change).chain(),
        );
    }
}

#[derive(Component, Default, Clone)]
pub struct Glyph {
    pub cp437: Option<char>,
    pub tile: Option<Tile>,
    pub fg1: Option<Color>,
    pub fg2: Option<Color>,
    pub bg: Option<Color>,
    pub outline: Option<Color>,
    pub is_shrouded: bool,
}

pub struct GlyphColors {
    pub fg1: Color,
    pub fg2: Color,
    pub bg: Color,
    pub outline: Color,
}

impl Glyph {
    pub fn get_cp437(&self) -> usize
    {
        match self.cp437 {
            Some(c) => cp437_idx(c).unwrap_or(0),
            None => CP437_NBSP,
        }
    }

    pub fn get_atlas_idx(&self) -> u32 {
        self.tile.unwrap_or(Tile::Dirt) as u32
    }

    pub fn get_colors(&self) -> GlyphColors {
        if self.is_shrouded {
            return GlyphColors {
                bg: TRANSPARENT,
                fg1: SHROUD_COLOR,
                fg2: SHROUD_COLOR,
                outline: CLEAR_COLOR,
            };
        }

        GlyphColors {
            bg: self.bg.unwrap_or(TRANSPARENT),
            fg1: self.fg1.unwrap_or(TRANSPARENT),
            fg2: self.fg2.unwrap_or(TRANSPARENT),
            outline: self.outline.unwrap_or(CLEAR_COLOR),
        }
    }
}

#[derive(Component, Clone, Copy)]
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

pub fn glyph_translation(x: usize, y: usize) -> Vec2 {
    let px = world_to_px(x, y);
    vec2(px.0 as f32, px.1 as f32)
}

pub fn add_glyph_material(
    mut cmds: Commands,
    q_glyphs: Query<(Entity, &Glyph), Added<Glyph>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GlyphMaterial>>,
    tileset: Res<Tileset>,
) {
    for (e, glyph) in q_glyphs.iter() {
        let colors = glyph.get_colors();
        let material = materials.add(GlyphMaterial {
            fg1: colors.fg1.into(),
            fg2: colors.fg2.into(),
            bg: colors.bg.into(),
            outline: colors.outline.into(),
            atlas: tileset.texture.clone_weak(),
            idx: glyph.get_atlas_idx(),
        });

        cmds.entity(e).insert((
            Mesh2d(meshes.add(Rectangle::from_size(vec2(TILE_SIZE_F32.0, TILE_SIZE_F32.1)))),
            MeshMaterial2d(material),
            Transform::default(),
        ));
    }
}

// update any sprites that have glyph changed
pub fn update_glyph_material(
    q_changed: Query<(&Glyph, &MeshMaterial2d<GlyphMaterial>), Changed<Glyph>>,
    mut materials: ResMut<Assets<GlyphMaterial>>,
) {
    for (glyph, mat_handle) in q_changed.iter() {
        let Some(material) = materials.get_mut(mat_handle) else {
            continue;
        };

        let colors = glyph.get_colors();

        material.fg1 = colors.fg1.into();
        material.fg2 = colors.fg2.into();
        material.bg = colors.bg.into();
        material.outline = colors.outline.into();
        material.idx = glyph.get_atlas_idx();
    }
}

pub fn update_positions(mut q_changed: Query<(&Position, &mut Transform), Changed<Position>>) {
    for (position, mut transform) in q_changed.iter_mut() {
        let z = (10 * position.z + (10 - position.layer)) as f32;
        let target = glyph_translation(position.x, position.y).extend(-z);
        transform.translation = target;
    }
}

#[derive(Resource, Default)]
pub struct Tileset {
    pub texture: Handle<Image>,
    pub font_texture: Handle<Image>,
}

pub fn setup_tileset(
    asset_server: Res<AssetServer>,
    mut tileset: ResMut<Tileset>,
) {
    tileset.texture = asset_server.load("cowboy.png");
    tileset.font_texture = asset_server.load("bizcat_8x12.png");
}

pub fn on_status_change(mut q_changed: Query<(&mut Glyph, &ZoneStatus), Changed<ZoneStatus>>) {
    for (mut glyph, status) in q_changed.iter_mut() {
        glyph.is_shrouded = *status == ZoneStatus::Dormant;
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GlyphMaterial {
    #[uniform(0)]
    fg1: LinearRgba,
    #[uniform(1)]
    fg2: LinearRgba,
    #[uniform(2)]
    bg: LinearRgba,
    #[uniform(3)]
    outline: LinearRgba,
    #[uniform(4)]
    idx: u32,
    #[texture(5)]
    #[sampler(6)]
    atlas: Handle<Image>,
}

impl Material2d for GlyphMaterial {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/glyph.wgsl".into()
    }
}
