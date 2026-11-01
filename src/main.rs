mod camera;
mod input;
mod level;
mod player;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::camera::*;
use crate::input::*;
use crate::level::*;
use crate::player::*;

fn main() {
    App::new()

        //Add plugins
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(level::LevelPlugin)
        .add_plugins(player::PlayerPlugin)
        //Annouce game start
        .add_systems(Startup, hello_game_system)
        // Spawn systems
        .add_systems(Startup, spawn_camera)
        //Input systems
        .add_systems(Update, capture_input)
        // Start game
        .run();
}

fn hello_game_system() {
    println!("Started cubes can roll!")
}
