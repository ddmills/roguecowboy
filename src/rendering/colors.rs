use bevy::color::{Alpha, Color};

use super::{Text, TextGlyph};

// pub const CLEAR_COLOR: Color = Color::srgb(0.02, 0.086, 0.153);
pub const SHROUD_COLOR: Color = Color::srgb(0.227, 0.243, 0.247);
pub const TRANSPARENT: Color = Color::srgba(0.659, 0.294, 0.294, 0.);
pub const TEXT_COLOR: Color = Color::srgb(0.804, 0.867, 0.875);

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Palette {
    White = 0xD2DBDB,
    Black = 0x171B1F,
    Green = 0x2E862E,
    LightGreen = 0x1AAF1A,
    Brown = 0x664D3C,
    Blue = 0x294E94,
    LightBlue = 0x608ED3,
    Red = 0xA83A3A,
    Orange = 0xE79519,
    Yellow = 0xEBCC21,
    Purple = 0xB31DB3,
    Cyan = 0x0EC9E2,
    DarkCyan = 0x2C7983,
}

pub fn hex(r: u8, g: u8, b: u8) -> u32
{
    ((r as u32) << 16) + ((g as u32) << 8) + (b as u32)
}

pub trait BevyColorable {
    fn to_bevy_color(&self) -> bevy::prelude::Color;
    fn to_color_a(&self, a:f32) -> bevy::prelude::Color;
}

impl BevyColorable for u32 {
    fn to_bevy_color(&self) -> bevy::prelude::Color {
        let b = (self & 0xff) as u8;
        let g = ((self >> 8) & 0xff) as u8;
        let r = ((self >> 16) & 0xff) as u8;

        bevy::prelude::Color::srgb_u8(r, g, b)
    }
    
    fn to_color_a(&self, a:f32) -> bevy::prelude::Color {
        let mut c = self.to_bevy_color();
        c.set_alpha(a);
        c
    }
}

impl BevyColorable for Palette {
    fn to_bevy_color(&self) -> bevy::prelude::Color {
        (*self as u32).to_bevy_color()
    }
    
    fn to_color_a(&self, a:f32) -> bevy::prelude::Color {
        (*self as u32).to_color_a(a)
    }
}

impl std::convert::From<Palette> for u32 {
    fn from(val: Palette) -> Self {
        val as u32
    }
}

pub const START_SEQ: char = '{';
pub const END_SEQ: char = '}';
pub const FLAG_SEQ: char = '|';

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
        "P" => Color::srgb(0.596, 0.051, 0.847),
        "p" => Color::srgb(0.412, 0.153, 0.533),
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
                bg: None,
                outline: None,
                tileset: super::Tileset::BodyFont,
            }
        }).collect()
    }
}

pub fn get_text_glyphs(text: &Text) -> Vec<TextGlyph>
{
    let mut in_seq = false;
    let mut in_flags = false;
    let mut seq_setting = String::new();
    let mut seq_value = String::new();

    text.value.chars().filter_map(|c| {
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
            fg1: text.fg1.map(|x| x.to_bevy_color()),
            fg2: text.fg2.map(|x| x.to_bevy_color()),
            bg: text.bg.map(|x| x.to_bevy_color()),
            outline: None,
            tileset: super::Tileset::BodyFont,
        }])
    }).flatten().collect()
}

#[test]
fn test_int_to_color() {
    let c = 0xff0a0b.to_bevy_color();
    let srgb = c.to_srgba();
    assert_eq!(srgb.red, 1.);
    assert_eq!(srgb.green, 0.039215688);
    assert_eq!(srgb.blue, 0.043137256);
}
