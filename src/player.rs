use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    GameState,
    glyph::{Glyph, Position},
    projection::{ZONE_SIZE, MAP_SIZE, Z_LAYER_ACTORS},
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
            idx: 2,
            fg: Color::srgb_u8(255, 0, 0),
        },
        Position::new(8, 8, 0, Z_LAYER_ACTORS),
    ));

    e_player_moved.send(PlayerMovedEvent { x: 8, y: 8, z: 0 });
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
    let rate = 0.035;
    let delay = 0.35;
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
