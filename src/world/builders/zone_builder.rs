use std::fmt::Display;

use bevy::color::Color;
use serde::{Deserialize, Serialize};

use crate::{common::Grid, world::Terrain};

pub const ENABLE_ZONE_SNAPSHOTS: bool = true;

#[derive(Deserialize, Serialize, Clone)]
pub struct ZoneData {
    pub idx: usize,
    pub terrain: Grid<Terrain>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TileSnapColor {
    #[default]
    White,
    Black,
    Gray(u8),
    Red,
    Blue,
    Green,
    Orange,
    Yellow,
}

impl Display for TileSnapColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl TileSnapColor {
    pub fn gray(v: f32) -> Self {
        let g = (v * 255.).floor() as u8;
        Self::Gray(g.clamp(0, 255))
    }

    pub fn name(self) -> String {
        match self {
            TileSnapColor::White => "White".into(),
            TileSnapColor::Black => "Black".into(),
            TileSnapColor::Gray(v) => format!("Gray ({})", v),
            TileSnapColor::Red => "Red".into(),
            TileSnapColor::Blue => "Blue".into(),
            TileSnapColor::Green => "Green".into(),
            TileSnapColor::Orange => "Orange".into(),
            TileSnapColor::Yellow => "Yellow".into(),
        }
    }

    pub fn for_terrain(t: Terrain) -> Self {
        match t {
            Terrain::Grass => Self::Green,
            Terrain::Dirt => Self::Orange,
            Terrain::River => Self::Blue,
            Terrain::Footpath => Self::Yellow,
        }
    }

    pub fn for_edge(e: u8) -> Self {
        match e {
            0 => Self::Gray(127),
            1 => Self::Blue,   // river
            2 => Self::Yellow, // footpath
            _ => Self::White,
        }
    }

    pub fn to_color(self) -> Color {
        match self {
            TileSnapColor::White => Color::WHITE,
            TileSnapColor::Black => Color::BLACK,
            TileSnapColor::Gray(p) => Color::srgb_u8(p, p, p),
            TileSnapColor::Red => Color::srgb(1., 0., 0.),
            TileSnapColor::Blue => Color::srgb(0., 0., 1.),
            TileSnapColor::Green => Color::srgb(0.255, 0.373, 0.255),
            TileSnapColor::Orange => Color::srgb(1., 0.5, 0.),
            TileSnapColor::Yellow => Color::srgb(1., 1., 0.),
        }
    }
}

#[derive(Clone)]
pub struct ZoneSnapshot {
    pub data: Grid<TileSnapColor>,
}

pub struct ZoneConstraints {
    pub idx: usize,
    pub south: Vec<u8>,
    pub west: Vec<u8>,
    pub east: Vec<u8>,
    pub north: Vec<u8>,
}

pub trait ZoneBuilder {
    fn build(&mut self, constraints: ZoneConstraints) -> ZoneData;
    fn get_snapshots(&self) -> Vec<ZoneSnapshot>;
}
