use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::{glyph::{Glyph, Position}, projection::{world_to_chunk_idx, CHUNK_SIZE, MAP_SIZE, Z_LAYER_ACTORS}, world::Chunks, GameState};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(OnEnter(GameState::Playing), (setup_player).chain())
            .add_systems(Update, (player_input, on_player_move).run_if(in_state(GameState::Playing)));
    }
}


#[derive(Component)]
pub struct Player;

pub fn setup_player(mut cmds: Commands) {
    cmds.spawn((
        Player,
        Glyph {
            idx: 2,
            fg: Color::srgb_u8(255, 0, 0),
            bg: Color::srgb_u8(0, 0, 255),
        },
        Position::new(8, 8, 0, Z_LAYER_ACTORS)
    ));
}

#[derive(Default)]
pub struct KeyState {
    time: f64,
    delayed: bool,
}

#[derive(Default)]
pub struct InputRate {
    keys: HashMap<KeyCode, KeyState>
}

impl InputRate {
    pub fn try_key(&mut self, key: KeyCode, now: f64, rate: f64, delay: f64) -> bool {
        if let Some(s) = self.keys.get(&key) {
            let t = match s.delayed {
                true => rate,
                false => delay,
            };

            if now - s.time > t {
                self.keys.insert(key, KeyState {
                    time: now,
                    delayed: true,
                });

                return true;
            }
            
            return false;
        };

        self.keys.insert(key, KeyState {
            time: now,
            delayed: false,
        });
        true
    }
}

pub fn player_input(
    mut q_player: Query<&mut Position, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut input_rate: Local<InputRate>
) {
    let now = time.elapsed_secs_f64();
    let rate = 0.075;
    let delay = 0.35;

    let mut position = q_player.single_mut();

    if position.y < (MAP_SIZE.1 * CHUNK_SIZE.1) - 1 && keys.pressed(KeyCode::KeyW) && input_rate.try_key(KeyCode::KeyW, now, rate, delay) {
        position.y += 1;
    }

    if position.x > 0 && keys.pressed(KeyCode::KeyA) && input_rate.try_key(KeyCode::KeyA, now, rate, delay) {
        position.x -= 1;
    }

    if position.y > 0 && keys.pressed(KeyCode::KeyS) && input_rate.try_key(KeyCode::KeyS, now, rate, delay) {
        position.y -= 1;
    }

    if position.x < (MAP_SIZE.0 * CHUNK_SIZE.0) - 1 && keys.pressed(KeyCode::KeyD) && input_rate.try_key(KeyCode::KeyD, now, rate, delay) {
        position.x += 1;
    }

    for key in keys.get_just_released() {
        input_rate.keys.remove(key);
    }
}


// check when player moves to a different chunk and set it as active
pub fn on_player_move(
    q_player: Query<&Position, With<Player>>,
    mut chunks: ResMut<Chunks>
) {
    let player = q_player.single();
    let player_chunk_idx = world_to_chunk_idx(player.x, player.y, player.z);

    // if !chunks.active.contains(&player_chunk_idx) {
    if player_chunk_idx == 0 {
        chunks.active = vec![0];
    } else {
        chunks.active = vec![0, player_chunk_idx];
    }
    // }
}
