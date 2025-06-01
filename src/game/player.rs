// filepath: /bevy-multiplayer-shooter/bevy-multiplayer-shooter/src/game/player.rs
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::AppState;

use super::camera::CameraControl;
use super::health::Health;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (player_align_to_camera_system, player_movement_system)
                .run_if(in_state(crate::AppState::Running)),
        );
    }
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub acceleration: f32,
}

impl Player {
    pub fn new(speed: f32, acceleration: f32) -> Self {
        Player { speed, acceleration }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Player (blauer Würfel)
    commands.spawn((
        Player { speed: 12.0, acceleration: 60.0 }, // <--- Beschleunigung hier setzen!
        Health { value: 100.0 },
        Mesh3d(meshes.add(Cuboid::new(1.2, 1.2, 1.2))),
        MeshMaterial3d(materials.add(Color::from(Srgba::new(0.2, 0.2, 1.0, 1.0)))),
        Transform::from_xyz(0.0, 0.6, 0.0),
        Visibility::Visible,
        RigidBody::Dynamic,
        Collider::cuboid(0.6, 0.6, 0.6),
        Velocity::zero(),
        ExternalForce::default(),
        ExternalImpulse::default(),
        Friction {
            coefficient: 0.1,
            combine_rule: CoefficientCombineRule::Average,
        },
        Restitution::default(),
        ColliderMassProperties::Density(2.0),
        (
            ActiveEvents::COLLISION_EVENTS,
            Damping {
                linear_damping: 0.5,
                angular_damping: 2.0,
            },
        ),
    ));
}

fn player_movement_system(
    input: Res<ButtonInput<KeyCode>>,
    camera_control: Res<crate::game::camera::CameraControl>,
    mut query: Query<(&Player, &mut Transform, &Velocity, &mut ExternalForce)>,
) {
    for (player, mut transform, velocity, mut force) in query.iter_mut() {
        let mut move_dir = Vec3::ZERO;

        // Kamera-Forward und Right auf XZ-Ebene berechnen
        let yaw = camera_control.yaw;
        let cam_forward = -Vec3::new(yaw.sin(), 0.0, yaw.cos()).normalize();
        let cam_right = Vec3::new(-cam_forward.z, 0.0, cam_forward.x);

        if input.pressed(KeyCode::KeyW) {
            move_dir += cam_forward;
        }
        if input.pressed(KeyCode::KeyS) {
            move_dir -= cam_forward;
        }
        if input.pressed(KeyCode::KeyA) {
            move_dir -= cam_right;
        }
        if input.pressed(KeyCode::KeyD) {
            move_dir += cam_right;
        }
        move_dir = move_dir.normalize_or_zero();

        let move_force = player.acceleration; // <-- jetzt aus Player struct!
        let max_speed = player.speed;

        let vel_in_dir = velocity.linvel.dot(move_dir);
        let force_in_dir = force.force.dot(move_dir);
        let force_rest = force.force - move_dir * force_in_dir;

        if vel_in_dir < max_speed {
            force.force = move_dir * move_force;
        } else {
            force.force = Vec3::ZERO;
        }
    }
}

fn player_align_to_camera_system(
    player_query: Single<(&Transform, &mut ExternalForce), With<crate::Player>>,
    camera_transform: Single<&Transform, (With<Camera3d>, Without<crate::Player>)>,
    time: Res<Time>,
) {
    let (player_transform, mut force) = player_query.into_inner();
    let camera_pos = camera_transform.translation;
    let player_pos = player_transform.translation;

    // Richtung von Kamera zum Spieler (nur XZ-Ebene)
    let dir = (player_pos - camera_pos).xz().normalize_or_zero();
    if dir.length_squared() == 0.0 {
        force.torque = Vec3::ZERO;
        return;
    }
    // Ziel-Yaw: Spieler soll von der Kamera wegschauen
    let target_yaw = f32::atan2(dir.x, dir.y);

    let current_yaw = player_transform.rotation.to_euler(EulerRot::YXZ).0;
    let mut delta = target_yaw - current_yaw;
    while delta > std::f32::consts::PI {
        delta -= 2.0 * std::f32::consts::PI;
    }
    while delta < -std::f32::consts::PI {
        delta += 2.0 * std::f32::consts::PI;
    }

    // Proportional zur Entfernung drehen, aber maximaler Wert begrenzen
    let turn_torque = 15.0;
    let max_torque = 60.0;
    let torque = (delta * turn_torque).clamp(-max_torque, max_torque);

    force.torque = Vec3::Y * torque;
}
