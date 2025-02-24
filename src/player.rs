use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    glyph::{Glyph, Position, Tile}, projection::{MAP_SIZE, ZONE_SIZE, Z_LAYER_ACTORS, Z_LAYER_SNAPSHOT, Z_LAYER_TEXT}, text::{GlyphText, TextPosition}, GameState
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_event::<PlayerMovedEvent>()
            .add_systems(Startup, (setup_player).chain())
            .add_systems(Update, (player_input).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Event)]
pub struct PlayerMovedEvent {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

pub fn setup_player(mut cmds: Commands, mut e_player_moved: EventWriter<PlayerMovedEvent>) {
    cmds.spawn((
        Player,
        Glyph {
            cp437: Some('@'),
            tile: Some(Tile::Cowboy),
            fg1: Some(Color::srgb_u8(255, 251, 11)),
            fg2: Some(Color::srgb_u8(181, 12, 223)),
            outline: None,
            bg: Some(Color::srgb_u8(151, 230, 99)),
            is_shrouded: false,
        },
        Position::new(8, 8, 0, Z_LAYER_ACTORS),
    ));

    e_player_moved.send(PlayerMovedEvent { x: 8, y: 8, z: 0 });

    cmds.spawn((
        GlyphText::new("Under the {C-b border|vast, starry sky}, the cowboy's {R|heart} ached"),
        TextPosition::new(4.0, 10.5, 0.0, Z_LAYER_TEXT),
    ));
    cmds.spawn((
        GlyphText::new("for new {y-Y-r-R-r-Y-y stretch|horizons} and {G-g-o-G-g-o repeat|untamed trails}."),
        TextPosition::new(4.0, 10.0, 0.0, Z_LAYER_TEXT),
    ));
    cmds.spawn((
        GlyphText::new("With a steady hand, you grip the {C-c-w-W-Y-C-c-C-w repeat|chrome-plated} pistol,"),
        TextPosition::new(4.0, 9.0, 0.0, Z_LAYER_TEXT),
    ));
    cmds.spawn((
        GlyphText::new("eyes scanning the {b|darkness}, ready to face the unknown."),
        TextPosition::new(4.0, 8.5, 0.0, Z_LAYER_TEXT),
    ));
}

#[derive(Default)]
pub struct KeyState {
    time: f64,
    delayed: bool,
}

#[derive(Default)]
pub struct InputRate {
    keys: HashMap<KeyCode, KeyState>,
}

impl InputRate {
    pub fn try_key(&mut self, key: KeyCode, now: f64, rate: f64, delay: f64) -> bool {
        if let Some(s) = self.keys.get(&key) {
            let t = match s.delayed {
                true => rate,
                false => delay,
            };

            if now - s.time > t {
                self.keys.insert(
                    key,
                    KeyState {
                        time: now,
                        delayed: true,
                    },
                );

                return true;
            }

            return false;
        };

        self.keys.insert(
            key,
            KeyState {
                time: now,
                delayed: false,
            },
        );
        true
    }
}

pub fn player_input(
    mut q_player: Query<&mut Position, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut input_rate: Local<InputRate>,
    mut e_player_moved: EventWriter<PlayerMovedEvent>,
) {
    let now = time.elapsed_secs_f64();
    let rate = 0.015;
    let delay = 0.25;
    let mut moved = false;

    let mut position = q_player.single_mut();

    if position.x > 0
        && keys.pressed(KeyCode::KeyA)
        && input_rate.try_key(KeyCode::KeyA, now, rate, delay)
    {
        position.x -= 1;
        moved = true;
    }

    if position.x < (MAP_SIZE.0 * ZONE_SIZE.0) - 1
        && keys.pressed(KeyCode::KeyD)
        && input_rate.try_key(KeyCode::KeyD, now, rate, delay)
    {
        position.x += 1;
        moved = true;
    }

    if position.y < (MAP_SIZE.1 * ZONE_SIZE.1) - 1
        && keys.pressed(KeyCode::KeyW)
        && input_rate.try_key(KeyCode::KeyW, now, rate, delay)
    {
        position.y += 1;
        moved = true;
    }

    if position.y > 0
        && keys.pressed(KeyCode::KeyS)
        && input_rate.try_key(KeyCode::KeyS, now, rate, delay)
    {
        position.y -= 1;
        moved = true;
    }

    if position.z > 0
        && keys.pressed(KeyCode::KeyE)
        && input_rate.try_key(KeyCode::KeyE, now, rate, delay)
    {
        position.z -= 1;
        moved = true;
    }

    if position.z < MAP_SIZE.2 - 1
        && keys.pressed(KeyCode::KeyQ)
        && input_rate.try_key(KeyCode::KeyQ, now, rate, delay)
    {
        position.z += 1;
        moved = true;
    }

    for key in keys.get_just_released() {
        input_rate.keys.remove(key);
    }

    if moved {
        e_player_moved.send(PlayerMovedEvent {
            x: position.x,
            y: position.y,
            z: position.z,
        });
    }
}
