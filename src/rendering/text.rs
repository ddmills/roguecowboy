use bevy::{prelude::*, render::render_resource::AsBindGroup, sprite::{AlphaMode2d, Material2d, Material2dPlugin}};

use crate::common::{cp437_idx, CP437_NBSP};

use super::{get_text_glyphs, BevyColorable, GlyphColors, Palette, Position, Tileset, TilesetTextures, SHROUD_COLOR, TRANSPARENT};


pub struct GlyphTextPlugin;

impl Plugin for GlyphTextPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .add_plugins(Material2dPlugin::<GlyphTextMaterial>::default())
            .add_systems(Update, (
                render_text,
                add_glyph_text_material,
                update_glyph_text_material
            ).chain());
    }
}

#[derive(Component)]
#[require(Position, Visibility)]
pub struct Text {
    pub value: String,
    pub bg: Option<u32>,
    pub fg1: Option<u32>,
    pub fg2: Option<u32>,
    pub tileset: Tileset,
}

impl Text {
    pub fn new(value: &str) -> Self
    {
        Self {
            value: value.into(),
            bg: None,
            fg1: Some(Palette::White.into()),
            fg2: None,
            tileset: Tileset::BodyFont,
        }
    }

    pub fn title(value: &str) -> Self
    {
        Self {
            value: value.into(),
            bg: None,
            fg1: Some(Palette::White.into()),
            fg2: None,
            tileset: Tileset::TitleFont,
        }
    }

    pub fn bg<T: Into<u32>>(mut self, bg: T) -> Self
    {
        self.bg = Some(bg.into());
        self
    }

    pub fn fg1<T: Into<u32>>(mut self, fg: T) -> Self
    {
        self.fg1 = Some(fg.into());
        self
    }

    pub fn fg2<T: Into<u32>>(mut self, fg2: T) -> Self
    {
        self.fg2 = Some(fg2.into());
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
    pub tileset: Tileset,
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

pub fn render_text(mut cmds: Commands, q_glyph_text: Query<(Entity, &Text), Added<Text>>) {
    for (entity, text) in q_glyph_text.iter() {
        for (idx, text_glyph) in get_text_glyphs(text).iter().enumerate() {
            let translation = text.tileset
                .get_translation_offset(idx as f32)
                .floor()
                .extend(0.0);

            let mut glyph = text_glyph.to_owned();
            glyph.tileset = text.tileset;
            glyph.bg = glyph.bg.or(text.bg.map(|x| x.to_bevy_color()));
            glyph.fg1 = glyph.fg1.or(text.fg1.map(|x| x.to_bevy_color()));
            glyph.fg2 = glyph.fg2.or(text.fg2.map(|x| x.to_bevy_color()));

            if glyph.tileset == Tileset::TitleFont {
                info!("offset y {}", translation.y);
            }

            let mut child = cmds.spawn((
                glyph,
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
    tileset: Res<TilesetTextures>,
) {
    for (e, glyph) in q_glyphs.iter() {
        let colors = glyph.get_colors();
        let atlas = tileset.get(glyph.tileset);
        let material = materials.add(GlyphTextMaterial {
            fg1: colors.fg1.into(),
            fg2: colors.fg2.into(),
            bg: colors.bg.into(),
            outline: colors.outline.into(),
            idx: glyph.get_cp437() as u32,
            atlas,
        });

        let size = glyph.tileset.get_size();

        cmds.entity(e).insert((
            Mesh2d(meshes.add(Rectangle::from_size(size))),
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
