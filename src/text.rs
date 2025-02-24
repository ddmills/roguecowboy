use std::default;

use bevy::{math::vec2, prelude::*, render::render_resource::AsBindGroup, sprite::{AlphaMode2d, Material2d, Material2dPlugin}};

use crate::{common::{cp437_idx, CP437_NBSP}, glyph::{Glyph, GlyphColors, Tileset, CLEAR_COLOR, TEXT_COLOR, TRANSPARENT}, projection::{TEXEL_SIZE_F32, TEXT_SIZE_F32, TEXT_TO_TILE_RATIO, TILE_SIZE, TILE_SIZE_F32, Z_LAYER_SNAPSHOT}};


pub struct GlyphTextPlugin;

impl Plugin for GlyphTextPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
            .add_plugins(Material2dPlugin::<GlyphTextMaterial>::default())
            .add_systems(Update, (on_spawn_text, add_glyph_text_material, update_text_glyph_positions, update_glyph_text_material).chain());
    }
}

#[derive(Component)]
#[require(TextPosition, Visibility)]
pub struct GlyphText {
    pub value: String,
}

const START_SEQ: char = '{';
const END_SEQ: char = '}';
const FLAG_SEQ: char = '|';

fn get_seq_color(ch:&str) -> Color
{
    match ch {
        "W" => TEXT_COLOR,
        "w" => Color::srgb(0.659, 0.659, 0.659),
        "R" => Color::srgb(0.878, 0.314, 0.314),
        "r" => Color::srgb(0.427, 0.153, 0.153),
        "G" => Color::srgb(0.365, 0.675, 0.184),
        "g" => Color::srgb(0.157, 0.286, 0.133),
        "B" => Color::srgb(0.067, 0.2, 0.941),
        "b" => Color::srgb(0.192, 0.31, 0.541),
        "Y" => Color::srgb(0.831, 0.827, 0.4),
        "y" => Color::srgb(0.655, 0.655, 0.22),
        "C" => Color::srgb(0.278, 0.769, 0.769),
        "c" => Color::srgb(0.263, 0.427, 0.467),
        "O" => Color::srgb(0.925, 0.643, 0.031),
        "o" => Color::srgb(0.467, 0.373, 0.2),
        _ => TEXT_COLOR,
    }
}

enum ColorSequenceType {
    Solid,
    Repeat,
    Stretch,
    Border,
}

impl ColorSequenceType {
    pub fn from_str(val: &str) -> ColorSequenceType {
        match val {
            "solid" => Self::Solid,
            "repeat" => Self::Repeat,
            "stretch" => Self::Stretch,
            "border" => Self::Border,
            _ => Self::Solid,
        }
    }
}

struct ColorSequence {
    seq_type: ColorSequenceType,
    seq_colors: Vec<Color>,
}

impl ColorSequence {
    pub fn new(value: String) -> Self
    {
        let split = value.split(' ').collect::<Vec<_>>();
        let mut seq_type = ColorSequenceType::Repeat;
        let mut seq_colors = value.clone();

        if split.len() == 2 {
            seq_type = ColorSequenceType::from_str(split[1]);
            seq_colors = split[0].to_string();
        }

        let mut colors = seq_colors
            .split('-')
            .map(get_seq_color)
            .collect::<Vec<_>>();

        if colors.is_empty() {
            colors = vec![TEXT_COLOR];
        }

        Self {
            seq_colors: colors,
            seq_type,
        }
    }

    pub fn apply_to(&mut self, value: String) -> Vec<TextGlyph>
    {
        let color_len = self.seq_colors.len();
        let value_len = value.len();

        value.chars().enumerate().map(|(idx, c)| {
            let fg1 = match self.seq_type {
                ColorSequenceType::Solid => *self.seq_colors.first().unwrap(),
                ColorSequenceType::Repeat => *self.seq_colors.get(idx % color_len).unwrap(),
                ColorSequenceType::Stretch => {
                    let dist = idx as f32 / value_len as f32;
                    let new_idx = (dist * color_len as f32).floor() as usize;
                    *self.seq_colors.get(new_idx).unwrap()
                },
                ColorSequenceType::Border => {
                    if idx == 0 || idx == value_len - 1 {
                        *self.seq_colors.first().unwrap()
                    } else {
                        *self.seq_colors.get(1 % color_len).unwrap()
                    }
                },
            };

            TextGlyph {
                cp437: Some(c),
                fg1: Some(fg1),
                fg2: None,
                bg: Some(CLEAR_COLOR),
                outline: None,
            }
        }).collect()
    }
}

impl GlyphText {
    pub fn new(value: &str) -> Self
    {
        Self {
            value: value.into()
        }
    }

    pub fn to_glyphs(&self) -> Vec<TextGlyph>
    {
        let default_fg = TEXT_COLOR;
        let mut in_seq = false;
        let mut in_flags = false;
        let mut seq_setting = String::new();
        let mut seq_value = String::new();

        self.value.chars().filter_map(|c| {
            if c == START_SEQ {
                in_seq = true;
                in_flags = true;
                return None;
            }

            if in_seq && c == END_SEQ {
                in_seq = false;
                in_flags = false;

                let mut seq = ColorSequence::new(seq_setting.clone());
                let glyphs= seq.apply_to(seq_value.clone());

                seq_setting = String::new();
                seq_value = String::new();

                return Some(glyphs);
            }

            if in_seq && c == FLAG_SEQ {
                in_flags = false;
                return None;
            }

            if in_flags {
                seq_setting.push(c);
                return None;
            }

            if in_seq {
                seq_value.push(c);
                return None;
            }

            Some(vec![TextGlyph {
                cp437: Some(c),
                fg1: Some(default_fg),
                fg2: None,
                // bg: None,
                bg: Some(CLEAR_COLOR),
                outline: None,
            }])
        }).flatten().collect()
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
            outline: self.outline.unwrap_or(CLEAR_COLOR),
        }
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct TextPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub layer: usize,
}

impl TextPosition {
    pub fn new(x: f32, y: f32, z: f32, layer: usize) -> Self {
        Self { x, y, z, layer }
    }
}

pub fn glyph_text_translation(x: f32, y: f32) -> Vec2 {
    let offset = (
        TEXT_SIZE_F32.0 * TEXT_TO_TILE_RATIO.0 * 0.5,
        TEXT_SIZE_F32.1 * TEXT_TO_TILE_RATIO.1 * 0.5
    );
    vec2((x * TILE_SIZE_F32.0) - offset.0, (y * TILE_SIZE_F32.1 * TEXT_TO_TILE_RATIO.1) - offset.1)
}

pub fn update_text_glyph_positions(mut q_changed: Query<(&TextPosition, &mut Transform), Changed<TextPosition>>) {
    for (position, mut transform) in q_changed.iter_mut() {
        let z = (200) as f32;

        let target = glyph_text_translation(position.x, position.y).extend(z);
        transform.translation = target;
    }
}

pub fn on_spawn_text(mut cmds: Commands, q_glyph_text: Query<(Entity, &TextPosition, &GlyphText), Added<GlyphText>>) {
    for (entity, position, text) in q_glyph_text.iter() {
        cmds.entity(entity).insert(Transform::default());

        for (idx, glyph) in text.to_glyphs().iter().enumerate() {
            let x_pos = idx as f32 * TEXT_TO_TILE_RATIO.0;
            let translation = glyph_text_translation(x_pos, position.y).extend(200.);

            let mut child =cmds.spawn((
                glyph.to_owned(),
                TextPosition::new(x_pos, position.y, position.z, Z_LAYER_SNAPSHOT),
                Transform::default().with_translation(translation),
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
            Transform::default(),
        ));
    }
}


pub fn update_glyph_text_material(
    q_changed: Query<(&TextGlyph, &MeshMaterial2d<GlyphTextMaterial>), Changed<Glyph>>,
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
