use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{camera::CameraFollowPriority, input::PlayerInput};


#[derive(Component)]
pub struct Player;

#[derive(Resource, Default)]
pub struct PlayerStats {
    move_toque: f32,
    player_dimensions: Vec3,
    max_angular_speed: f32
}
#[derive(Component, Default)]
pub struct PlayerState {
    is_grounded: bool,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInput>()
            .init_resource::<PlayerStats>()
            .add_systems(Startup, spawn_player)
            .add_systems(FixedUpdate, (check_grounded, move_player).chain());
    }
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_stats: ResMut<PlayerStats>,
) {
    player_stats.player_dimensions = vec3(1.0, 1.0, 1.0);
    player_stats.move_toque = 10.0;
    player_stats.max_angular_speed = 5.0;

    commands.spawn((
        Player,
        RigidBody::Dynamic,
        Friction::new(10.0),
        PlayerState::default(),
        MaxAngularSpeed(5.0),
        CollidingEntities::default(),
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
        CameraFollowPriority(5)
        
    ));
}

pub fn move_player(
    player_input: Res<PlayerInput>,
    player_stats: Res<PlayerStats>,
    mut players: Query<(Forces, &PlayerState, &mut MaxAngularSpeed), With<Player>>,
) {
    for (mut forces, player_state, mut max_angular_speed) in &mut players {

        if player_input.sprint_pressed{
            max_angular_speed.0 = player_stats.max_angular_speed * 2.0
        } else {
            max_angular_speed.0 = player_stats.max_angular_speed
        }
        
        if player_state.is_grounded {
            let move_direction = Vec3::new(player_input.movement.x, 0.0, player_input.movement.y);
            // Torque perpendicular to the movement direction.
            let torque = Vec3::Y.cross(move_direction) * player_stats.move_toque;
            forces.apply_torque(torque);
        }
    }
}

pub fn check_grounded(
    mut players: Query<(&CollidingEntities, &mut PlayerState), With<Player>>,
) {
    for (colliding, mut player_state) in &mut players {
        // FUTURE: When triggers or other colliders are added that should not count towards grounded state, filter them out here
        player_state.is_grounded = !colliding.0.is_empty();
    }
}
