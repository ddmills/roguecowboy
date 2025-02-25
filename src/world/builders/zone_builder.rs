use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{common::Grid, rendering::{hex, Palette}, world::Terrain};

pub const ENABLE_ZONE_SNAPSHOTS: bool = false;

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

    pub fn to_color(self) -> u32 {
        match self {
            TileSnapColor::White => Palette::White.into(),
            TileSnapColor::Black => Palette::Black.into(),
            TileSnapColor::Gray(p) => hex(p, p, p).into(),
            TileSnapColor::Red => Palette::Red.into(),
            TileSnapColor::Blue => Palette::Blue.into(),
            TileSnapColor::Green => Palette::Green.into(),
            TileSnapColor::Orange => Palette::Orange.into(),
            TileSnapColor::Yellow => Palette::Yellow.into(),
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
