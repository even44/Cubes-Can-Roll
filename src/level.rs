use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
struct LevelEntity;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LevelId {
    Default,
}

#[derive(Resource, Default)]
struct RequestedLevel(Option<LevelId>);

#[derive(Resource, Default)]
struct CurrentLevel(Option<LevelId>);

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RequestedLevel>()
            .init_resource::<CurrentLevel>()
            .add_systems(Startup, request_default_level)
            .add_systems(Update, load_requested_level);
    }
}

fn request_default_level(mut request: ResMut<RequestedLevel>) {
    request.0 = Some(LevelId::Default);
}

fn load_requested_level(
    mut commands: Commands,
    mut requested: ResMut<RequestedLevel>,
    mut current: ResMut<CurrentLevel>,
    old_level_entities: Query<Entity, With<LevelEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(next) = requested.0.take() else {
        return;
    };

    if current.0 == Some(next) {
        return;
    }

    for entity in &old_level_entities {
        commands.entity(entity).despawn();
    }

    match next {
        LevelId::Default => {
            // Ground to sit on
            commands.spawn((
                LevelEntity,
                RigidBody::Static,
                Collider::cuboid(20.0, 1.0, 20.0),
                Mesh3d(meshes.add(Cuboid::new(20.0, 1.0, 20.0))),
                MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
                Transform::from_xyz(0.0, -0.5, 0.0),
            ));
        }
    }

    current.0 = Some(next);
}
