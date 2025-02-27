use bevy::prelude::*;

use crate::{projection::{TILE_SIZE, TILE_SIZE_F32, Z_LAYER_TEXT}, rendering::{BevyColorable, Glyph, Palette, Position, Text, Tile}};


pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, render_box);
    }
}

#[derive(Component)]
pub struct UiBox {
    pub width: usize,
    pub height: usize,
    pub title: Option<String>,
    pub icon: Option<Glyph>,
}

impl UiBox {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            title: None,
            icon: None,
        }
    }

    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn icon(mut self, icon: Glyph) -> Self {
        self.icon = Some(icon);
        self
    }
}

pub fn render_box(
    mut cmds: Commands,
    q_ui_boxes: Query<(Entity, &UiBox), (With<Transform>, Added<UiBox>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    for (entity, settings) in q_ui_boxes.iter() {
        let width = (settings.width * TILE_SIZE.0) as f32;
        let height = (settings.height * TILE_SIZE.1) as f32;
        let rect = meshes.add(Rectangle::new(width, height));

        let color = Palette::Black.to_bevy_color();

        // background rectangle
        cmds.spawn((
            Mesh2d(rect),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(
                width / 2. - TILE_SIZE_F32.0 / 2.,
                height / 2. - TILE_SIZE_F32.1 / 2., 500.),
            )).set_parent(entity);

        let z_layer = Z_LAYER_TEXT + 1000;

        if let Some(title) = &settings.title {
            cmds.spawn((
                Text::title(title).bg(Palette::Black).fg1(Palette::Yellow),
                Position::f32(3.0, (settings.height - 1) as f32, 0.0, z_layer + 1),
            )).set_parent(entity);
        }

        if let Some(icon) = &settings.icon {
            cmds.spawn((
                // Glyph::new(Tile::Cowboy, Palette::Yellow, Palette::Green).bg(Palette::Black),
                icon.clone(),
                Position::new(2, settings.height - 1, 0, z_layer + 1),
            )).set_parent(entity);
        }

        for y in 1..(settings.height - 1) {
            cmds.spawn((
                Glyph::new(Tile::BoxLeft, Palette::Blue, Palette::Orange),
                Position::new(0, y, 0, z_layer - 1),
            )).set_parent(entity);

            cmds.spawn((
                Glyph::new(Tile::BoxRight, Palette::Blue, Palette::Orange),
                Position::new(settings.width - 1, y, 0, z_layer - 1),
            )).set_parent(entity);
        }

        for x in 1..(settings.width - 1) {
            cmds.spawn((
                Glyph::new(Tile::BoxBottom, Palette::Blue, Palette::Orange),
                Position::new(x, 0, 0, z_layer - 1),
            )).set_parent(entity);

            cmds.spawn((
                Glyph::new(Tile::BoxTop, Palette::Blue, Palette::Orange),
                Position::new(x, settings.height - 1, 0, z_layer),
            )).set_parent(entity);
        }

        cmds.spawn((
            Glyph::new(Tile::BoxTopLeft, Palette::Blue, Palette::Orange),
            Position::new(0, settings.height - 1, 0, z_layer - 1),
        )).set_parent(entity);

        cmds.spawn((
            Glyph::new(Tile::BoxTopRight, Palette::Blue, Palette::Orange),
            Position::new(settings.width - 1, settings.height - 1, 0, z_layer - 1),
        )).set_parent(entity);

        cmds.spawn((
            Glyph::new(Tile::BoxBottomLeft, Palette::Blue, Palette::Orange),
            Position::new(0, 0, 0, z_layer - 1),
        )).set_parent(entity);

        cmds.spawn((
            Glyph::new(Tile::BoxBottomRight, Palette::Blue, Palette::Orange),
            Position::new(settings.width - 1, 0, 0, z_layer - 1),
        )).set_parent(entity);
    }
}
