use avian3d::prelude::*;
use bevy::prelude::*;

use crate::player::{self, Player};

#[derive(Component)]
struct LevelEntity;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LevelId {
    Default,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct SpawnPoint;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Finish;

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
            .add_systems(Update, load_requested_level)
            .add_systems(Update, place_player_at_spawn)
            .add_systems(Update, detect_finish);
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
    asset_server: Res<AssetServer>,
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
                WorldAssetRoot(
                    asset_server.load(GltfAssetLabel::Scene(0).from_asset("levels/test_level.glb")),
                ),
                ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            ));
        }
    }

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
    mut reached: Local<bool>,
) {
    for &position in players {
        for &finish in finishes {
            if position.0.distance(finish.translation()) < 1.0 {
                if !*reached {
                    info!("Finish reached!");
                    *reached = true;
                }
            }
        }
    }
}
