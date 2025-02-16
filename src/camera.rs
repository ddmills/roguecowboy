use bevy::{app::{App, Plugin, Startup}, core_pipeline::core_2d::Camera2d, ecs::system::Commands};


pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut cmds: Commands) {
    cmds.spawn(Camera2d);
}
