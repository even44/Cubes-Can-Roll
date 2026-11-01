use bevy::{input::mouse::{AccumulatedMouseMotion}, prelude::*};

#[derive(Resource, Default)]
pub struct PlayerInput {
    pub mouse_delta: Vec2,
    pub movement: Vec2,
    pub jump_pressed: bool,
    pub sprint_pressed: bool
}

pub fn capture_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<AccumulatedMouseMotion>,
    mut player_input: ResMut<PlayerInput>,
) {
    let mut movement = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        movement.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        movement.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        movement.x += 1.0;
    }

    player_input.mouse_delta = mouse.delta;
    player_input.movement = movement.normalize_or_zero();
    player_input.jump_pressed = keys.just_pressed(KeyCode::Space);
}
