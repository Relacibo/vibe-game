// filepath: /bevy-multiplayer-shooter/bevy-multiplayer-shooter/src/game/player.rs
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

impl Player {
    pub fn new(speed: f32) -> Self {
        Player { speed }
    }
}

pub fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut player: Single<(
        &Player,
        &mut Transform,
        &mut Velocity,
        &mut ExternalImpulse,
        &mut ExternalForce,
    )>,
) {
    let sensitivity = 0.1;
    let max_speed = 4.0;
    let move_force = 30.0;

    let (player, transform, mut velocity, mut impulse, mut force) = player.into_inner();
    // Rotation wie gehabt
    let delta = mouse_motion.delta;
    velocity.angvel = Vec3::Y * (-delta.x * sensitivity);

    let mut direction = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.z -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.z += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction != Vec3::ZERO {
        let world_dir = transform.rotation * direction.normalize();
        let vel_in_dir = velocity.linvel.dot(world_dir);
        if vel_in_dir < max_speed {
            force.force = world_dir * move_force;
        } else {
            force.force = Vec3::ZERO;
        }
    } else {
        force.force = Vec3::ZERO;
    }

    // Dash (Shift)
    if keyboard_input.just_pressed(KeyCode::ShiftLeft)
        || keyboard_input.just_pressed(KeyCode::ShiftRight)
    {
        let dash_strength = 20.0;
        let forward = transform.forward();
        impulse.impulse += forward * dash_strength;
    }
    // Jump (Space)
    if keyboard_input.just_pressed(KeyCode::Space) {
        let jump_strength = 15.0;
        impulse.impulse += Vec3::Y * jump_strength;
    }
}
