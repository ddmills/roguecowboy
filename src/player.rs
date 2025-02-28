use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    camera::Layer, projection::{MAP_SIZE, ZONE_SIZE, Z_LAYER_ACTORS, Z_LAYER_TEXT}, rendering::{Glyph, Palette, Position, Text, Tile}, ui::UiBox, GameState
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
        Glyph::new(Tile::Cowboy, Palette::Yellow, Palette::Purple),
        Position::new(8, 8, 0, Layer::Actors),
    ));

    e_player_moved.send(PlayerMovedEvent { x: 8, y: 8, z: 0 });

    cmds.spawn((
        Text::new(" You discovered... ").bg(Palette::Black).fg1(Palette::White),
        Position::f32(17.0, 15., 0.0, Layer::Ui),
    ));

    cmds.spawn((
        Text::title("♦ {C|ESPERLOOSA OUTPOST} ♦").bg(Palette::Black).fg1(Palette::Yellow).fg2(Palette::Cyan),
        Position::f32(16.0, 14., 0.0, Layer::Ui),
    ));
    cmds.spawn((
        Text::new("Under the {C-b border|vast, starry sky}, the {R-O-Y-G-B-P stretch|cowboy's} {R|heart} ached").bg(Palette::Black),
        Position::f32(4.0, 6.0, 0.0, Layer::Fx),
    ));
    cmds.spawn((
        Text::new("for new {r-R-Y-Y-Y-Y-R-r stretch|horizons} and {G-g-o-G-g-o repeat|untamed trails}.").bg(Palette::Black),
        Position::f32(4.0, 10.0, 0.0, Layer::Fx),
    ));
    cmds.spawn((
        Text::new("With a steady hand, you grip the {C-c-w-W-Y-C-c-C-w repeat|chrome-plated pistol},").bg(Palette::Black),
        Position::f32(4.0, 9.0, 0.0, Layer::Fx),
    ));
    cmds.spawn((
        Text::new("Eyes Scanning The {b|Darkness}, Ready to Face the Unknown.").bg(Palette::Black),
        Position::f32(4.0, 8.5, 0.0, Layer::Fx),
    ));
    cmds.spawn((
        Text::new("{R-O-Y-G-B-P stretch|Howdy Cowboy!}").bg(Palette::Black),
        Position::f32(0.0, 3.0, 0.0, Layer::Fx),
    ));
    cmds.spawn((
        Text::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ").bg(Palette::Black),
        Position::f32(18.0, 3.5, 0.0, Layer::Ui),
    ));
    cmds.spawn((
        Text::new("abcdefghijklmnopqrstuvwxyz").bg(Palette::Black),
        Position::f32(18.0, 3.0, 0.0, Layer::Ui),
    ));
    cmds.spawn((
        Text::new("You don't always have to be a {R-O-Y-G-B-P stretch|strong} cowboy.").bg(Palette::Black),
        Position::f32(0.0, 0.5, 0.0, Layer::Fx),
    ));
    cmds.spawn((
        Text::new("sometimes just being an {R-O-Y-G-B-P stretch|alive} cowboy is enough.").bg(Palette::Black),
        Position::f32(0.0, 0.0, 0.0, Layer::Fx), // TODO: maybe use `Position` component for text as well, pass a flag alon, Layer::Fxg.
    ));
    cmds.spawn((
        UiBox::new(24, 12)
            .title("Inventory")
            .icon(Glyph::new(Tile::Cowboy, Palette::Purple, Palette::LightGreen)),
        Position::new(3, 44, 0, Layer::Fx),
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
    let rate = 0.020;
    let delay = 0.25;
    let mut moved = false;

    let mut position = q_player.single_mut();
    let (x, y, z) = position.world();

    if x > 0
        && keys.pressed(KeyCode::KeyA)
        && input_rate.try_key(KeyCode::KeyA, now, rate, delay)
    {
        position.x(x - 1);
        moved = true;
    }

    if x < (MAP_SIZE.0 * ZONE_SIZE.0) - 1
        && keys.pressed(KeyCode::KeyD)
        && input_rate.try_key(KeyCode::KeyD, now, rate, delay)
    {
        position.x(x + 1);
        moved = true;
    }

    if y < (MAP_SIZE.1 * ZONE_SIZE.1) - 1
        && keys.pressed(KeyCode::KeyW)
        && input_rate.try_key(KeyCode::KeyW, now, rate, delay)
    {
        position.y(y + 1);
        moved = true;
    }

    if y > 0
        && keys.pressed(KeyCode::KeyS)
        && input_rate.try_key(KeyCode::KeyS, now, rate, delay)
    {
        position.y(y - 1);
        moved = true;
    }

    if z > 0
        && keys.pressed(KeyCode::KeyE)
        && input_rate.try_key(KeyCode::KeyE, now, rate, delay)
    {
        position.z(z - 1);
        moved = true;
    }

    if z < MAP_SIZE.2 - 1
        && keys.pressed(KeyCode::KeyQ)
        && input_rate.try_key(KeyCode::KeyQ, now, rate, delay)
    {
        position.z(z + 1);
        moved = true;
    }

    for key in keys.get_just_released() {
        input_rate.keys.remove(key);
    }

    if moved {
        e_player_moved.send(PlayerMovedEvent {
            x: position.x as usize,
            y: position.y as usize,
            z: position.z as usize,
        });
    }
}
