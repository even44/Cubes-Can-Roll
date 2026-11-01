use bevy::prelude::*;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.0, 8.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
