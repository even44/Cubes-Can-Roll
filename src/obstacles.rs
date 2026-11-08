use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{level::LevelEntity, player::Player};

#[derive(Reflect, Debug)]
enum CubePileMode {
    Count { count: u32 },
    Density { density: f32 },
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct CubePile {
    mode: CubePileMode,
    min_size: f32,
    max_size: f32,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Spinner {
    speed: f32,
    arms: u32,
    #[reflect(default = "return_spinner_default_axis")]
    axis: Vec3,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Bouncer {
    impulse: f32, // fixed kick, e.g. 8.0
    #[reflect(default)]
    up: f32, // extra upward pop, e.g. 2.0
}

pub struct ObstaclesPlugin;
impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_cube_piles)
            .add_systems(Update, spawn_spinners)
            .add_systems(Update, setup_bounce_observer);
    }
}

fn spawn_cube_piles(
    mut commands: Commands,
    piles: Query<(&CubePile, &GlobalTransform), Added<CubePile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (pile, transform) in &piles {
        if pile.min_size <= 0.0 || pile.max_size < pile.min_size {
            warn!("CubePile needs 0 < min_size <= max_size");
            continue;
        }

        let center = transform.translation();
        let half = transform.scale();
        let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let material = materials.add(Color::srgb(0.9, 0.5, 0.2));

        // Each cube gets a grid cell big enough for the largest size.
        let cell = pile.max_size * 1.1;
        let columns = ((half.x * 2.0 / cell) as u32).max(1);
        let rows = ((half.z * 2.0 / cell) as u32).max(1);

        let count = match pile.mode {
            CubePileMode::Count { count } => count,
            CubePileMode::Density { density } => {
                let wanted = density_count(density, half, pile.min_size, pile.max_size);
                let layers = ((half.y * 2.0 / cell) as u32).max(1);
                let capacity = columns * rows * layers;
                if wanted > capacity {
                    warn!(
                        "CubePile density {density} needs {wanted} cubes, but only {capacity} fit"
                    );
                }
                wanted.min(capacity)
            }
        };

        let mut cubes = Vec::with_capacity(count as usize);

        let grid_start = Vec3::new(
            -(columns as f32) * cell / 2.0,
            -half.y,
            -(rows as f32) * cell / 2.0,
        );

        for i in 0..count {
            let column = i % columns;
            let row = (i / columns) % rows;
            let layer = i / (columns * rows);

            let position = center
                + grid_start
                + cell * Vec3::new(column as f32 + 0.5, layer as f32 + 0.5, row as f32 + 0.5);
            let size = pile.min_size + fastrand::f32() * (pile.max_size - pile.min_size);

            cubes.push((
                LevelEntity,
                RigidBody::Dynamic,
                Collider::cuboid(1.0, 1.0, 1.0),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(position).with_scale(Vec3::splat(size)),
            ));
        }

        commands.spawn_batch(cubes);
    }
}

fn density_count(density: f32, half: Vec3, min: f32, max: f32) -> u32 {
    let volume = 8.0 * half.x * half.y * half.z;
    let average_cube_volume = (min + max) * (min * min + max * max) / 4.0;
    (density * volume / average_cube_volume).round() as u32
}

fn spawn_spinners(
    mut commands: Commands,
    spinners: Query<(&Spinner, &GlobalTransform), Added<Spinner>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (spinner, transform) in &spinners {
        let (scale, rotation, translation) = transform.to_scale_rotation_translation();
        let radius = scale.y;
        let length = ((scale.x - radius) * 2.0).max(0.0);
        let mesh = meshes.add(Capsule3d::new(radius, length));
        let material = materials.add(Color::srgb(0.8, 0.2, 0.2));
        let mut axis = spinner.axis.normalize_or_zero();
        if axis == Vec3::ZERO {
            warn!("Spinner with axis Vec3::ZERO defaulting to Vec3::Y");
            axis = Vec3::Y
        }
        let spawn_rotation = rotation * Quat::from_rotation_arc(Vec3::Y, axis);

        commands
            .spawn((
                LevelEntity,
                RigidBody::Kinematic,
                AngularVelocity((spawn_rotation * Vec3::Y) * spinner.speed),
                Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
                Transform::from_translation(translation).with_rotation(spawn_rotation),
                Visibility::default(),
            ))
            .with_children(|parent| {
                for i in 0..spinner.arms.max(1) {
                    let angle = std::f32::consts::PI * i as f32 / spinner.arms.max(1) as f32;
                    parent.spawn((
                        Collider::capsule(radius, length),
                        Bouncer{
                            impulse: 8.0,
                            up: 1.0
                        },
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_rotation(
                            Quat::from_rotation_y(angle)
                                * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                        ),
                    ));
                }
            });
    }
}

fn setup_bounce_observer(
    mut commands: Commands,
    new_colliders: Query<Entity, Added<Collider>>,
    parents: Query<&ChildOf>,
    bouncers: Query<(), With<Bouncer>>,
) {
    // Colliders come from ColliderConstructorHierarchy, so they live on mesh
    // children of the entity that carries Bouncer (or on it directly).
    for collider in new_colliders {
        if bouncers.contains(collider)
            || parents.iter_ancestors(collider).any(|e| bouncers.contains(e))
        {
            info!("bouncer on {collider}");
            commands
                .entity(collider)
                .insert(CollisionEventsEnabled)
                .observe(on_bouncer_hit);
        }
    }
}

fn on_bouncer_hit(
    event: On<CollisionStart>,
    collisions: Collisions,
    bouncers: Query<&Bouncer>,
    parents: Query<&ChildOf>,
    mut players: Query<Forces, With<Player>>,
) {
    let bouncer_entity = event.collider1; // the observer's target
    let player_entity = event.collider2;

    let Ok(mut forces) = players.get_mut(player_entity) else {
        return;
    };
    // Bouncer sits on the collider itself or on one of its ancestors.
    let Some(bouncer) = std::iter::once(bouncer_entity)
        .chain(parents.iter_ancestors(bouncer_entity))
        .find_map(|e| bouncers.get(e).ok())
    else {
        return;
    };
    let Some(pair) = collisions.get(bouncer_entity, player_entity) else {
        return;
    };
    let Some(manifold) = pair.manifolds.first() else {
        return;
    };

    // Manifold normal points collider1 -> collider2; we want bouncer -> player.
    let normal = if pair.collider1 == bouncer_entity {
        manifold.normal
    } else {
        -manifold.normal
    };

    forces.apply_linear_impulse(normal * bouncer.impulse + Vec3::Y * bouncer.up);
}

fn return_spinner_default_axis() -> Vec3 {
    Vec3::Y
}
