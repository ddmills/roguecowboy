use std::default;

use bevy::{math::vec2, prelude::*, render::render_resource::AsBindGroup, sprite::{AlphaMode2d, Material2d, Material2dPlugin}};

use crate::{
    projection::{world_to_zone_idx, TEXT_SIZE, TEXT_SIZE_F32, TILE_SIZE_F32, TITLE_SIZE_F32}, world::ZoneStatus
};

use super::{BevyColorable, Palette, SHROUD_COLOR, TRANSPARENT};


#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Grass = 3,
    Water = 34,
    Cowboy = 146,
    Dirt = 19,
    Blank = 238,
    BoxTopRight = 223,
    BoxTop = 222,
    BoxTopLeft = 221,
    BoxLeft = 237,
    BoxRight = 239,
    BoxBottomLeft = 253,
    BoxBottom = 254,
    BoxBottomRight = 255,
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
#[require(Position)]
pub struct Glyph {
    pub tile: Option<Tile>,
    pub fg1: Option<u32>,
    pub fg2: Option<u32>,
    pub bg: Option<u32>,
    pub outline: Option<u32>,
    pub is_shrouded: bool,
}

pub struct GlyphColors {
    pub fg1: Color,
    pub fg2: Color,
    pub bg: Color,
    pub outline: Color,
}

impl Glyph {
    pub fn new<T: Into<u32>>(tile: Tile, fg1: T, fg2: T) -> Self
    {
        Self {
            tile: Some(tile),
            fg1: Some(fg1.into()),
            fg2: Some(fg2.into()),
            bg: None,
            outline: None,
            is_shrouded: false,
        }
    }

    pub fn bg<T: Into<u32>>(mut self, bg: T) -> Self
    {
        self.bg = Some(bg.into());
        self
    }

    pub fn outline<T: Into<u32>>(mut self, outline: T) -> Self
    {
        self.outline = Some(outline.into());
        self
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
                outline: Palette::Black.to_bevy_color(),
            };
        }

        GlyphColors {
            bg: self.bg.map(|x| x.to_bevy_color()).unwrap_or(TRANSPARENT),
            fg1: self.fg1.map(|x| x.to_bevy_color()).unwrap_or(TRANSPARENT),
            fg2: self.fg2.map(|x| x.to_bevy_color()).unwrap_or(TRANSPARENT),
            outline: self.outline.map(|x| x.to_bevy_color()).unwrap_or(Palette::Black.to_bevy_color()),
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
#[require(Transform, Visibility)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub layer: usize,
}

impl Position {
    pub fn new(x: usize, y: usize, z: usize, layer: usize) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            z: z as f32,
            layer
        }
    }

    pub fn f32<T: Into<f32>>(x: T, y: T, z: T, layer: usize) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
            layer
        }
    }
    
    #[inline]
    pub fn x(&mut self, x: usize) {
        self.x = x as f32;
    }
    
    #[inline]
    pub fn y(&mut self, y: usize) {
        self.y = y as f32;
    }
    
    #[inline]
    pub fn z(&mut self, z: usize) {
        self.z = z as f32;
    }

    #[inline]
    pub fn world(&self) -> (usize, usize, usize)
    {
        (self.x as usize, self.y as usize, self.z as usize)
    }

    #[inline]
    pub fn zone_idx(&self) -> usize
    {
        world_to_zone_idx(
            self.x as usize,
            self.y as usize,
            self.z as usize,
        )
    }
}

pub fn glyph_translation(x: f32, y: f32) -> Vec2 {
    vec2(x * TILE_SIZE_F32.0, y * TILE_SIZE_F32.1)
}

pub fn add_glyph_material(
    mut cmds: Commands,
    q_glyphs: Query<(Entity, &Glyph), Added<Glyph>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GlyphMaterial>>,
    tileset: Res<TilesetTextures>,
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
        let z = 100. * position.z + (100. - position.layer as f32);
        let target = glyph_translation(position.x, position.y).extend(-z);
        transform.translation = target;
    }
}

#[derive(Resource, Default)]
pub struct TilesetTextures {
    pub texture: Handle<Image>,
    pub font_texture: Handle<Image>,
    pub font_title_texture: Handle<Image>,
}

impl TilesetTextures {
    pub fn get(&self, tileset: Tileset) -> Handle<Image>
    {
        match tileset {
            Tileset::Sprite => self.texture.clone_weak(),
            Tileset::BodyFont => self.font_texture.clone_weak(),
            Tileset::TitleFont => self.font_title_texture.clone_weak(),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Tileset {
    #[default]
    Sprite,
    BodyFont,
    TitleFont,
}

impl Tileset {
    pub fn get_size(&self) -> Vec2
    {
        match self {
            Tileset::Sprite => vec2(TILE_SIZE_F32.0, TILE_SIZE_F32.1),
            Tileset::BodyFont => vec2(TEXT_SIZE_F32.0, TEXT_SIZE_F32.1),
            Tileset::TitleFont => vec2(TITLE_SIZE_F32.0, TITLE_SIZE_F32.1),
        }
    }

    pub fn get_translation_offset(&self, x: f32) -> Vec2
    {
        match self {
            Tileset::Sprite => vec2(TILE_SIZE_F32.0, TILE_SIZE_F32.1),
            Tileset::BodyFont => vec2(
                ((x * TEXT_SIZE_F32.0) - TEXT_SIZE_F32.0 / 2.).floor(),
                (-TEXT_SIZE_F32.1 / 2.).floor(),
            ),
            Tileset::TitleFont => vec2(
                ((x * TITLE_SIZE_F32.0) - TITLE_SIZE_F32.0 / 2.).floor(),
                0.0,
            ),
        }
    }
}

pub fn setup_tileset(
    asset_server: Res<AssetServer>,
    mut tileset: ResMut<TilesetTextures>,
) {
    tileset.texture = asset_server.load("cowboy.png");
    tileset.font_texture = asset_server.load("sans_8x12.png");
    tileset.font_title_texture = asset_server.load("nix8810_8x24.png");
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
