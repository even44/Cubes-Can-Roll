use avian3d::prelude::*;
use bevy::prelude::*;

use crate::player::Player;

const LEVELS: &[&str] = &["levels/test_level.glb", "levels/test_level_02.glb"];
const KILL_HEIGHT: f32 = -10.0;
#[derive(Component)]
pub struct LevelEntity;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct SpawnPoint;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Finish;

#[derive(Resource, Default)]
struct RequestedLevel(Option<usize>);

#[derive(Resource, Default)]
struct CurrentLevel(Option<usize>);

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RequestedLevel>()
            .init_resource::<CurrentLevel>()
            .add_systems(Startup, request_default_level)
            .add_systems(
                Update,
                (
                    place_player_at_spawn,
                    detect_finish,
                    restart_level,
                    load_requested_level,
                )
                    .chain(),
            );
    }
}

fn request_default_level(mut request: ResMut<RequestedLevel>) {
    request.0 = Some(0);
}

fn load_requested_level(
    mut commands: Commands,
    mut requested: ResMut<RequestedLevel>,
    mut current: ResMut<CurrentLevel>,
    old_level_entities: Query<Entity, With<LevelEntity>>,
    asset_server: Res<AssetServer>,
) {
    let Some(next) = requested.0.take() else {
        return;
    };

    for entity in &old_level_entities {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        LevelEntity,
        RigidBody::Static,
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(LEVELS[next]))),
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
    ));

    current.0 = Some(next);
}

fn place_player_at_spawn(
    spawns: Query<&GlobalTransform, Added<SpawnPoint>>,
    mut players: Query<
        (
            &mut Position,
            &mut Rotation,
            &mut LinearVelocity,
            &mut AngularVelocity,
        ),
        With<Player>,
    >,
) {
    let Some(spawn) = spawns.iter().next() else {
        return;
    };

    for (mut position, mut rotation, mut linear, mut angular) in &mut players {
        position.0 = spawn.translation();
        *rotation = Rotation::default();
        linear.0 = Vec3::ZERO;
        angular.0 = Vec3::ZERO;
    }
}

fn detect_finish(
    finishes: Query<&GlobalTransform, With<Finish>>,
    players: Query<&Position, With<Player>>,
    current: Res<CurrentLevel>,
    mut requested: ResMut<RequestedLevel>,
) {
    let Some(index) = current.0 else { return };
    for &position in players {
        for &finish in finishes {
            if position.0.distance(finish.translation()) < 1.0 {
                info!("Finish reached!");
                requested.0 = Some((index + 1) % LEVELS.len());
            }
        }
    }
}

fn restart_level(
    keys: Res<ButtonInput<KeyCode>>,
    players: Query<&Position, With<Player>>,
    spawn_points: Query<(), With<SpawnPoint>>,
    current: Res<CurrentLevel>,
    mut requested: ResMut<RequestedLevel>,
) {
    let fell =
        !spawn_points.is_empty() && players.iter().any(|position| position.0.y < KILL_HEIGHT);

    if fell || keys.just_pressed(KeyCode::KeyR) {
        requested.0 = current.0;
    }
}
