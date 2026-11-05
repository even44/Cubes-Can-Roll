use avian3d::physics_transform::Position;
use bevy::{prelude::*, render::view::Tonemapping};

#[derive(Component)]
pub struct CameraFollowPriority(pub u32);

#[derive(Component)]
pub struct CameraFollow {
    offset: Vec3,
}
impl CameraFollow {
    pub fn new(offset: Vec3) -> Self {
        Self { offset }
    }
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        CameraFollow::new(Vec3::new(0.0, 3.0, 7.0)),
        Transform::from_xyz(0.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        Tonemapping::AgX,
    ));
}

pub fn follow_target(
    targets: Query<(&CameraFollowPriority, &Position), With<CameraFollowPriority>>,
    mut camera: Single<
        (&mut Transform, &CameraFollow),
        (With<CameraFollow>, Without<CameraFollowPriority>),
    >,
) {
    let (camera_transform, camera_follow) = &mut *camera;

    let highest_target = targets.iter().max_by_key(|(priority, _)| priority.0);

    if let Some((_, target_position)) = highest_target {
        camera_transform.translation = target_position.0 + camera_follow.offset;
        camera_transform.look_at(target_position.0, Vec3::Y)
    }
}
