mod camera;
mod input;
mod level;
mod obstacles;
mod player;

use avian3d::PhysicsPlugins;
use avian3d::dynamics::rigid_body::Friction;
use avian3d::dynamics::rigid_body::Restitution;
use avian3d::dynamics::rigid_body::mass_properties::components::ColliderDensity;
use avian3d::dynamics::rigid_body::mass_properties::components::Mass;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_skein::SkeinPlugin;

use crate::camera::*;
use crate::input::*;

fn main() {
    App::new()
        //Add plugins
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(SkeinPlugin::default())
        .register_type::<ColliderDensity>()
        .register_type::<Mass>()
        .register_type::<Restitution>()
        .register_type::<Friction>()
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_plugins(level::LevelPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(obstacles::ObstaclesPlugin)
        //Annouce game start
        .add_systems(Startup, hello_game_system)
        // Spawn systems
        .add_systems(Startup, spawn_camera)
        //Lighting
        .add_systems(Update, set_sun_settings)
        //Camera
        .add_systems(Update, follow_target)
        //Input systems
        .add_systems(Update, capture_input)
        // Start game
        .run();
}

fn hello_game_system() {
    println!("Started cubes can roll!")
}

fn set_sun_settings(mut suns: Query<&mut DirectionalLight, Added<DirectionalLight>>) {
    for mut sun in &mut suns {
        sun.shadow_maps_enabled = true;
        sun.illuminance = 10_000.0
    }
}
