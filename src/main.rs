mod camera;
mod input;
mod level;
mod obstacles;
mod player;

use avian3d::prelude::*;
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
        .add_plugins(level::LevelPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(obstacles::ObstaclesPlugin)
        //Annouce game start
        .add_systems(Startup, hello_game_system)
        // Spawn systems
        .add_systems(Startup, spawn_camera)
        //Lighting
        .add_systems(Update, enable_sun_shadows)
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

fn enable_sun_shadows(mut suns: Query<&mut DirectionalLight, Added<DirectionalLight>>) {
    for mut sun in &mut suns {
        sun.shadow_maps_enabled = true;
    }
}
