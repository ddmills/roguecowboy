use bevy::{math::vec2, prelude::*, render::render_resource::AsBindGroup, sprite::{AlphaMode2d, Material2d, Material2dPlugin}};

use crate::{common::{cp437_idx, CP437_NBSP}, projection::TEXT_SIZE_F32};

use super::{get_text_glyphs, BevyColorable, GlyphColors, Palette, Position, Tileset, END_SEQ, FLAG_SEQ, START_SEQ, TEXT_COLOR, TRANSPARENT};


pub struct GlyphTextPlugin;

impl Plugin for GlyphTextPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .add_plugins(Material2dPlugin::<GlyphTextMaterial>::default())
            .add_systems(Update, (render_text, add_glyph_text_material, update_glyph_text_material).chain());
    }
}

#[derive(Component)]
#[require(Position, Visibility)]
pub struct Text {
    pub value: String,
    pub bg: Option<u32>,
    pub fg: Option<u32>,
}

impl Text {
    pub fn new(value: &str) -> Self
    {
        Self {
            value: value.into(),
            bg: None,
            fg: Some(Palette::White.into()),
        }
    }

    pub fn bg(mut self, bg: u32) -> Self
    {
        self.bg = Some(bg);
        self
    }

    pub fn fg(mut self, fg: u32) -> Self
    {
        self.fg = Some(fg);
        self
    }
}


#[derive(Component, Default, Clone)]
pub struct TextGlyph {
    pub cp437: Option<char>,
    pub fg1: Option<Color>,
    pub fg2: Option<Color>,
    pub bg: Option<Color>,
    pub outline: Option<Color>,
}

impl TextGlyph {
    pub fn get_cp437(&self) -> usize
    {
        match self.cp437 {
            Some(c) => cp437_idx(c).unwrap_or(0),
            None => CP437_NBSP,
        }
    }

    pub fn get_colors(&self) -> GlyphColors {
        GlyphColors {
            bg: self.bg.unwrap_or(TRANSPARENT),
            fg1: self.fg1.unwrap_or(TRANSPARENT),
            fg2: self.fg2.unwrap_or(TRANSPARENT),
            outline: self.outline.unwrap_or(Palette::Black.to_bevy_color()),
        }
    }
}

pub fn glyph_text_translation(idx: usize) -> Vec2 {
    vec2(
        (idx as f32 * TEXT_SIZE_F32.0) - TEXT_SIZE_F32.0 / 2.,
        (TEXT_SIZE_F32.1 / 2.) - TEXT_SIZE_F32.1,
    )
}

pub fn render_text(mut cmds: Commands, q_glyph_text: Query<(Entity, &Text), Added<Text>>) {
    for (entity, text) in q_glyph_text.iter() {
        for (idx, text_glyph) in get_text_glyphs(text).iter().enumerate() {
            let translation = glyph_text_translation(idx).extend(0.0);

            let mut child = cmds.spawn((
                text_glyph.to_owned(),
                Transform::from_translation(translation),
            ));

            child.set_parent(entity);
        }
    }
}

pub fn add_glyph_text_material(
    mut cmds: Commands,
    q_glyphs: Query<(Entity, &TextGlyph), Added<TextGlyph>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GlyphTextMaterial>>,
    tileset: Res<Tileset>,
) {
    for (e, glyph) in q_glyphs.iter() {
        let colors = glyph.get_colors();
        let material = materials.add(GlyphTextMaterial {
            fg1: colors.fg1.into(),
            fg2: colors.fg2.into(),
            bg: colors.bg.into(),
            outline: colors.outline.into(),
            idx: glyph.get_cp437() as u32,
            atlas: tileset.font_texture.clone_weak(),
        });

        cmds.entity(e).insert((
            Mesh2d(meshes.add(Rectangle::from_size(vec2(TEXT_SIZE_F32.0, TEXT_SIZE_F32.1)))),
            MeshMaterial2d(material),
        ));
    }
}

pub fn update_glyph_text_material(
    q_changed: Query<(&TextGlyph, &MeshMaterial2d<GlyphTextMaterial>), Changed<TextGlyph>>,
    mut materials: ResMut<Assets<GlyphTextMaterial>>,
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
        material.idx = glyph.get_cp437() as u32;
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GlyphTextMaterial {
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

impl Material2d for GlyphTextMaterial {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/glyph.wgsl".into()
    }
}
