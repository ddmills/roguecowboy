use bevy::prelude::*;

use crate::{glyph::{Glyph, Position}, projection::{world_to_chunk_idx, CHUNK_SIZE, MAP_SIZE, Z_LAYER_ACTORS}, world::{Chunk, Chunks, Map, OverworldChunk}};


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

pub fn player_input(
    mut q_player: Query<&mut Position, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut position = q_player.single_mut();

    if position.y < (MAP_SIZE.1 * CHUNK_SIZE.1) - 1 && keys.just_pressed(KeyCode::KeyW) {
        position.y += 1;
    }

    if position.x > 0 && keys.just_pressed(KeyCode::KeyA) {
        position.x -= 1;
    }

    if position.y > 0 && keys.just_pressed(KeyCode::KeyS) {
        position.y -= 1;
    }

    if position.x < (MAP_SIZE.0 * CHUNK_SIZE.0) - 1 && keys.just_pressed(KeyCode::KeyD) {
        position.x += 1;
    }
}

// check when player moves to a different chunk and set it as active
pub fn on_player_move(
    q_player: Query<&Position, With<Player>>,
    mut chunks: ResMut<Chunks>
) {
    let player = q_player.single();
    let player_chunk_idx = world_to_chunk_idx(player.x, player.y, player.z);

    if player_chunk_idx != chunks.active_idx {
        chunks.active_idx = player_chunk_idx;
    }
}
