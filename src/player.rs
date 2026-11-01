use avian3d::prelude::*;
use bevy::prelude::*;

use crate::input::PlayerInput;

#[derive(Component)]
pub struct Player;

#[derive(Resource, Default)]
pub struct PlayerStats {
    move_force: f32,
    player_dimensions: Vec3,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInput>()
            .init_resource::<PlayerStats>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, move_player);
    }
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_stats: ResMut<PlayerStats>,
) {
    player_stats.player_dimensions = vec3(1.0, 1.0, 1.0);
    player_stats.move_force = 5.0;

    commands.spawn((
        Player,
        RigidBody::Dynamic,
        Collider::cuboid(
            player_stats.player_dimensions.x,
            player_stats.player_dimensions.y,
            player_stats.player_dimensions.z,
        ),
        Mesh3d(meshes.add(Cuboid::new(
            player_stats.player_dimensions.x,
            player_stats.player_dimensions.y,
            player_stats.player_dimensions.z,
        ))),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 144, 144))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

pub fn move_player(
    player_input: Res<PlayerInput>,
    player_stats: Res<PlayerStats>,
    mut forces: Query<Forces, With<Player>>,
) {
    let move_direction = vec3(player_input.movement.x, 0.0, player_input.movement.y);
    for mut force in &mut forces {
        force.apply_force(move_direction * player_stats.move_force);
    }
}
