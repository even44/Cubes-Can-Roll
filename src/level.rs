use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{camera::SKY_COLOR, player::Player};

//const LEVELS: &[&str] = &["levels/level_01.glb", "levels/level_02.glb", "levels/level_03.glb", "levels/level_04.glb", "levels/level_05.glb", "levels/level_06.glb"];
const LEVELS: &[&str] = &["levels/test_level.glb"];
const KILL_HEIGHT: f32 = -10.0;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LevelPhase {
    #[default]
    Playing,
    Results,
}

#[derive(PhysicsLayer, Default)]
pub enum Layer { #[default] Default, Player, Trigger }

#[derive(Message, Debug, Clone, Copy)]
pub struct LevelFinished {
    pub index: usize,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct LevelLoaded {
    pub index: usize,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct LevelRestarted {
    pub index: usize,
}

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct SpawnPoint;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Finish;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct NoCollision;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct KillZone;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct KitPreview;

#[derive(Resource, Default)]
struct RequestedLevel(Option<usize>);

#[derive(Resource, Default)]
struct CurrentLevel(Option<usize>);

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RequestedLevel>()
            .init_resource::<CurrentLevel>()
            .insert_resource(ClearColor(SKY_COLOR))
            .add_message::<LevelFinished>()
            .add_message::<LevelLoaded>()
            .add_message::<LevelRestarted>()
            .init_state::<LevelPhase>()
            .add_systems(Startup, request_default_level)
            .add_systems(OnExit(LevelPhase::Results), advance_to_next_level)
            .add_systems(
                Update,
                (
                    place_player_at_spawn,
                    hide_kit_previews,
                    setup_finish_collider,
                    setup_killzone_collider,
                    remove_nocollision_colliders,
                    remove_preview_colliders,
                    restart_level.run_if(in_state(LevelPhase::Playing)),
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
    mut level_loaded: MessageWriter<LevelLoaded>,
    mut players: Query<
        (
            &mut Position,
            &mut Rotation,
            &mut LinearVelocity,
            &mut AngularVelocity,
        ),
        With<Player>,
    >,
    current_level: Res<CurrentLevel>,
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
    let Some(index) = current_level.0 else { return };
    level_loaded.write(LevelLoaded { index });
}

fn remove_nocollision_colliders(
    mut commands: Commands,
    new_colliders: Query<Entity, Added<Collider>>,
    parents: Query<&ChildOf>,
    no_collisions: Query<(), With<NoCollision>>,
) {
    for collider in new_colliders {
        if parents
            .iter_ancestors(collider)
            .any(|e| no_collisions.contains(e))
        {
            commands.entity(collider).remove::<Collider>();
        }
    }
}

fn remove_preview_colliders(
    mut commands: Commands,
    new_colliders: Query<Entity, Added<Collider>>,
    parents: Query<&ChildOf>,
    kit_previews: Query<(), With<KitPreview>>,
) {
    for collider in new_colliders {
        if parents
            .iter_ancestors(collider)
            .any(|e| kit_previews.contains(e))
        {
            commands.entity(collider).remove::<Collider>();
        }
    }
}

fn setup_finish_collider(
    mut commands: Commands,
    new_colliders: Query<Entity, Added<Collider>>,
    parents: Query<&ChildOf>,
    finishes: Query<(), With<Finish>>,
) {
    for collider in new_colliders {
        if parents
            .iter_ancestors(collider)
            .any(|e| finishes.contains(e))
        {
            info!("finish sensor on {collider}");
            commands
                .entity(collider)
                .insert((Sensor, CollisionEventsEnabled, CollisionLayers::new(Layer::Trigger, LayerMask::ALL)))
                .observe(on_finish_entered);
        }
    }
}

fn on_finish_entered(
    event: On<CollisionStart>,
    players: Query<(), With<Player>>,
    current: Res<CurrentLevel>,
    phase: Res<State<LevelPhase>>,
    mut next: ResMut<NextState<LevelPhase>>,
    mut finished: MessageWriter<LevelFinished>,
) {
    if *phase.get() != LevelPhase::Playing {
        return;
    }

    let collider2 = event.collider2;
    info!("Collider2 {collider2}");
    if !players.contains(event.collider2) {
        return;
    }
    let Some(index) = current.0 else { return };
    finished.write(LevelFinished { index });
    next.set(LevelPhase::Results);
}

fn setup_killzone_collider(
    mut commands: Commands,
    new_colliders: Query<Entity, Added<Collider>>,
    parents: Query<&ChildOf>,
    killzones: Query<(), With<KillZone>>,
) {
    for collider in new_colliders {
        if parents
            .iter_ancestors(collider)
            .any(|e| killzones.contains(e))
        {
            info!("killzone sensor on {collider}");
            commands
                .entity(collider)
                .insert((Sensor, CollisionEventsEnabled,CollisionLayers::new(Layer::Trigger, LayerMask::ALL)))
                .observe(on_killzone_entered);
        }
    }
}

fn hide_kit_previews(mut commands: Commands, previews: Query<Entity, Added<KitPreview>>) {
    for preview in previews {
        commands.entity(preview).insert(Visibility::Hidden);
    }
}

fn on_killzone_entered(
    event: On<CollisionStart>,
    players: Query<(), With<Player>>,
    current: Res<CurrentLevel>,
    mut requested: ResMut<RequestedLevel>,
    mut level_restarted: MessageWriter<LevelRestarted>,
) {
    if !players.contains(event.collider2) {
        return;
    }
    info!("You died!");
    requested.0 = current.0;
    let Some(index) = current.0 else { return };
    level_restarted.write(LevelRestarted { index });
}

fn restart_level(
    keys: Res<ButtonInput<KeyCode>>,
    players: Query<&Position, With<Player>>,
    spawn_points: Query<(), With<SpawnPoint>>,
    current: Res<CurrentLevel>,
    mut requested: ResMut<RequestedLevel>,
    mut level_restarted: MessageWriter<LevelRestarted>,
) {
    let Some(index) = current.0 else { return };

    let fell =
        !spawn_points.is_empty() && players.iter().any(|position| position.0.y < KILL_HEIGHT);

    if fell || keys.just_pressed(KeyCode::KeyR) {
        requested.0 = current.0;
        level_restarted.write(LevelRestarted { index });
    }
}

fn advance_to_next_level(current: Res<CurrentLevel>, mut requested: ResMut<RequestedLevel>) {
    let Some(index) = current.0 else { return };
    requested.0 = Some((index + 1) % LEVELS.len());
}
