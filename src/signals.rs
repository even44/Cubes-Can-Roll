use avian3d::collision::collider::{Collider, CollisionLayers, LayerMask, Sensor};
use avian3d::collision::collision_events::{CollisionEnd, CollisionEventsEnabled, CollisionStart};
use avian3d::dynamics::rigid_body::RigidBody;
use bevy::platform::collections::HashSet;

use bevy::prelude::*;

use crate::level::Layer;
use crate::player::Player;

#[derive(Component, Default)]
pub struct Active(pub bool);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(Active)]
struct Receiver {
    channel: u32,
}

#[derive(Reflect, Debug, Default)]
enum PressedBy {
    #[default]
    AnyDynamic,
    PlayerOnly,
}

#[derive(Component, Default)]
struct Occupants(HashSet<Entity>);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(Occupants)]
struct Sender {
    channel: u32,
    #[reflect(default)]
    pressed_by: PressedBy,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct PressZone;

#[derive(Resource, Default)]
struct ChannelTable(HashSet<u32>);

pub struct SignalsPlugin;
impl Plugin for SignalsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChannelTable>()
            .add_systems(Update, setup_press_zone_collider)
            .add_systems(
                Update,
                (
                    prune_occupants,
                    update_channel_table,
                    update_receivers,
                    log_channel_table,
                )
                    .chain(),
            );
    }
}

fn log_new_sender(senders: Query<&Sender, Added<Sender>>) {
    for sender in senders {
        let channel = sender.channel;
        info!("Got sender with channel {channel}")
    }
}

fn log_new_receiver(receivers: Query<&Receiver, Added<Receiver>>) {
    for receiver in receivers {
        let channel = receiver.channel;
        info!("Got Receiver with channel {channel}")
    }
}
fn log_channel_table(table: Res<ChannelTable>, mut last: Local<HashSet<u32>>) {
    if table.0 != *last {
        info!("Hot channels: {:?}", table.0);
        *last = table.0.clone();
    }
}
fn update_channel_table(mut table: ResMut<ChannelTable>, senders: Query<(&Sender, &Occupants)>) {
    table.0.clear();
    for (sender, occupants) in &senders {
        if !occupants.0.is_empty() {
            //Add channel to set
            table.0.insert(sender.channel);
        }
    }
}

fn prune_occupants(mut occupants: Query<&mut Occupants>, alive: Query<()>) {
    for mut occupants in &mut occupants {
        occupants.0.retain(|e| alive.contains(*e));
    }
}

fn update_receivers(table: Res<ChannelTable>, mut receivers: Query<(&Receiver, &mut Active)>) {
    for (receiver, mut active) in &mut receivers {
        active.0 = table.0.contains(&receiver.channel);
    }
}

fn setup_press_zone_collider(
    mut commands: Commands,
    new_colliders: Query<Entity, Added<Collider>>,
    parents: Query<&ChildOf>,
    press_zones: Query<(), With<PressZone>>,
    senders: Query<(), With<Sender>>,
) {
    for collider in new_colliders {
        if parents
            .iter_ancestors(collider)
            .any(|e| press_zones.contains(e))
        {
            if !parents
                .iter_ancestors(collider)
                .any(|e| senders.contains(e))
            {
                warn!("PressZone on {collider} has no Sender above it");
                continue;
            }
            info!("press zone sensor on {collider}");
            commands
                .entity(collider)
                .insert((
                    Sensor,
                    CollisionEventsEnabled,
                    CollisionLayers::new(Layer::Trigger, LayerMask::ALL),
                ))
                .observe(on_press_zone_entered)
                .observe(on_press_zone_exited);
        }
    }
}

fn on_press_zone_entered(
    event: On<CollisionStart>,
    parents: Query<&ChildOf>,
    bodies: Query<&RigidBody>,
    players: Query<(), With<Player>>,
    mut senders: Query<(&Sender, &mut Occupants)>,
) {
    let Some(sender_entity) = parents
        .iter_ancestors(event.collider1)
        .find(|e| senders.contains(*e))
    else {
        return;
    };
    let Some(body) = event.body2 else { return };
    let Ok((sender, mut occupants)) = senders.get_mut(sender_entity) else {
        return;
    };

    let allowed = match sender.pressed_by {
        PressedBy::AnyDynamic => bodies.get(body).is_ok_and(|b| b.is_dynamic()),
        PressedBy::PlayerOnly => players.contains(body),
    };
    if allowed {
        occupants.0.insert(event.collider2);
    }
}

fn on_press_zone_exited(
    event: On<CollisionEnd>,
    parents: Query<&ChildOf>,
    mut occupants: Query<&mut Occupants>,
) {
    let Some(sender_entity) = parents
        .iter_ancestors(event.collider1)
        .find(|e| occupants.contains(*e))
    else {
        return;
    };
    if let Ok(mut occupants) = occupants.get_mut(sender_entity) {
        occupants.0.remove(&event.collider2);
    }
}
