use avian3d::{
    collision::collider::Collider,
    physics_transform::Position,
    spatial_query::{ShapeCastConfig, SpatialQuery, SpatialQueryFilter},
};
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    render::view::Tonemapping,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::level::Layer;

pub const SKY_COLOR: Color = Color::srgb(0.62, 0.78, 0.93);

#[derive(Component)]
pub struct CameraFollowPriority(pub u32);

#[derive(Component)]
pub struct OrbitCamera {
    pub yaw: f32,              // radians, around Y
    pub pitch: f32,            // radians, clamped
    pub distance: f32,         // desired arm length (e.g. 6.0)
    pub current_distance: f32, // what the arm is actually at this frame
    pub sensitivity: f32,
    pub target_offset: Vec3, // look a bit above the cube, e.g. (0, 0.5, 0)
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        OrbitCamera {
            yaw: 0.0,
            pitch: -0.32,
            distance: 6.3,
            current_distance: 6.3,
            sensitivity: 0.003,
            target_offset: Vec3::Y / 2.0,
        },
        Transform::from_xyz(0.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        Tonemapping::AgX,
        Projection::Perspective(PerspectiveProjection {
            fov: 45f32.to_radians(),
            ..default()
        }),
        DistanceFog {
            color: SKY_COLOR,
            falloff: FogFalloff::Linear {
                start: 60.0,
                end: 180.0,
            },
            ..default()
        },
    ));
}

pub fn orbit_target(
    targets: Query<(&CameraFollowPriority, &Position, Entity), With<CameraFollowPriority>>,
    mut camera: Single<
        (&mut Transform, &mut OrbitCamera),
        (With<OrbitCamera>, Without<CameraFollowPriority>),
    >,
    mouse_input: Res<AccumulatedMouseMotion>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
) {
    let (camera_transform, orbit_camera) = &mut *camera;

    let highest_target = targets.iter().max_by_key(|(priority, _, _)| priority.0);

    if let Some((_, target_position, target)) = highest_target {
        if cursor.grab_mode == CursorGrabMode::Locked {
            orbit_camera.yaw -= mouse_input.delta.x * orbit_camera.sensitivity;
            orbit_camera.pitch -= mouse_input.delta.y * orbit_camera.sensitivity;
        }
        orbit_camera.pitch = orbit_camera.pitch.clamp(-1.4, 0.2);
        let pivot = target_position.0 + orbit_camera.target_offset;
        let rotation = Quat::from_euler(EulerRot::YXZ, orbit_camera.yaw, orbit_camera.pitch, 0.0);
        let dir = rotation * Vec3::Z;

        let Ok(dir3) = Dir3::new(dir) else { return };
        let probe = Collider::sphere(0.25); // ~camera near-plane "thickness"
        let config = ShapeCastConfig::from_max_distance(orbit_camera.distance);
        let filter = SpatialQueryFilter::from_mask(Layer::Default);
       

        let wanted = spatial_query
            .cast_shape(&probe, pivot, Quat::IDENTITY, dir3, &config, &filter)
            .map(|hit| hit.distance) // sphere centre stops at the wall
            .unwrap_or(orbit_camera.distance)
            .max(0.5); // never go inside the cube

        // Spring: snap in, ease out
        if wanted < orbit_camera.current_distance {
            orbit_camera.current_distance = wanted;
        } else {
            orbit_camera
                .current_distance
                .smooth_nudge(&wanted, 6.0, time.delta_secs());
        }

        camera_transform.translation = pivot + dir * orbit_camera.current_distance;
        camera_transform.look_at(pivot, Vec3::Y); // .distance = how far the shape travelled before touching
    }
}

pub fn grab_cursor(
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
    if keys.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}
