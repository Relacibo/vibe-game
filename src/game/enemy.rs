use crate::{AppState, Ground, Health, Player};
use bevy::{audio, prelude::*, state::commands};
use bevy_rapier3d::{na::RealField, prelude::*};
use rand::Rng;

use super::bullet::{Bullet, BulletLifetime};

#[derive(Resource, Clone)]
struct EnemyShootSound(Handle<AudioSource>);

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    enemy_movement_system,
                    enemy_shooting,
                    enemy_despawn_far_system,
                    maybe_spawn_enemy.run_if(enemy_count_under_threshold),
                )
                    .run_if(in_state(AppState::Running)),
            )
            .insert_resource(EnemySpawnDelay {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            });
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let enemy_shoot_sound = asset_server.load("sounds/arrow_shoot.wav");
    commands.insert_resource(EnemyShootSound(enemy_shoot_sound));
}

#[derive(Component)]
pub struct Enemy {
    pub damage: f32,
    pub material: Handle<StandardMaterial>,
    pub max_turn_speed: f32,
    pub move_force: f32,
    pub max_speed: f32,
    pub turn_torque: f32,
}

fn enemy_movement_system(
    time: Res<Time>,
    player_query: Single<&Transform, With<Player>>,
    mut enemy_query: Query<(
        &Enemy,
        &Transform,
        &Velocity,
        &mut EnemyMovement,
        &mut ExternalForce,
    )>,
) {
    let player_pos = player_query.translation;

    for (enemy, enemy_transform, velocity, mut movement, mut force) in enemy_query.iter_mut() {
        // Prüfe, ob Gegner auf dem Boden ist (Y-Position nahe 0.5)
        if (enemy_transform.translation.y - 0.5).abs() > 0.06 {
            // Nicht auf dem Boden: Keine Bewegungskraft anwenden!
            continue;
        }

        // --- Rotation ---
        let enemy_pos = enemy_transform.translation;
        let to_player = Vec3::new(player_pos.x, enemy_pos.y, player_pos.z) - enemy_pos;
        let forward = enemy_transform.forward();
        let target_dir = to_player.normalize_or_zero();

        if target_dir.length_squared() > 0.0 {
            let angle = forward.angle_between(target_dir);
            let axis = forward.cross(target_dir).normalize_or_zero();
            let max_angle = enemy.max_turn_speed * time.delta_secs();
            let clamped_angle = angle.min(max_angle);

            if clamped_angle > 0.001 && axis.length_squared() > 0.0 {
                let sign = axis.y.signum();
                force.torque = Vec3::Y * sign * enemy.turn_torque * clamped_angle;
            } else {
                force.torque = Vec3::ZERO;
            }
        } else {
            force.torque = Vec3::ZERO;
        }

        // --- Zufällige Bewegungsrichtung ---
        movement.change_timer.tick(time.delta());
        if movement.change_timer.just_finished() {
            let mut rng = rand::rng();

            // Richtung zum Spieler (nur XZ-Ebene)
            let to_player = (player_pos - enemy_transform.translation).normalize_or_zero();
            let player_dir = Vec3::new(to_player.x, 0.0, to_player.z).normalize_or_zero();

            // Gewichtung: 70% Richtung Spieler, 30% bisherige Richtung
            let weight_player = 0.8;
            let weight_old = 1.0 - weight_player;
            let mut new_dir =
                (player_dir * weight_player + movement.direction * weight_old).normalize_or_zero();

            // Kleine Zufallsrotation (z.B. -0.3 bis 0.3 Bogenmaß)
            let angle = rng.random_range(-0.3..0.3);
            let rot = Quat::from_rotation_y(angle);
            movement.direction = (rot * new_dir).normalize_or_zero();

            // Timer auf neuen Zufallswert setzen (z.B. 1.0 bis 3.0 Sekunden)
            movement.change_timer =
                Timer::from_seconds(rng.random_range(1.0..3.0), TimerMode::Once);
        }

        // --- Bewegung mit Force ---
        let world_dir = enemy_transform.rotation * movement.direction;
        let vel_in_dir = velocity.linvel.dot(world_dir);

        let force_in_dir = force.force.dot(world_dir);
        let force_rest = force.force - world_dir * force_in_dir;

        if vel_in_dir < enemy.max_speed {
            force.force = force_rest + world_dir * enemy.move_force;
        } else {
            force.force = force_rest;
        }
    }
}

fn enemy_despawn_far_system(
    mut commands: Commands,
    player_query: Single<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    let player_pos = player_query.translation;
    for (entity, transform) in enemy_query.iter() {
        if (transform.translation - player_pos).length() > 300.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn maybe_spawn_enemy(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_delay: ResMut<EnemySpawnDelay>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Single<&Transform, With<Player>>,
) {
    let mut rng = rand::rng();
    spawn_delay.timer.tick(time.delta());
    if spawn_delay.timer.just_finished() {
        let player_pos = player_query.translation;
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let dist = rng.random_range(20.0..40.0);
        let x = player_pos.x + dist * angle.cos();
        let z = player_pos.z + dist * angle.sin();

        let enemy_health = rng.random_range(50.0..=150.0);
        let enemy_damage = rng.random_range(5.0..=20.0);

        let enemy_material = materials.add(Color::from(Srgba::new(1.0, 0.2, 0.2, 1.0)));
        commands.spawn((
            Enemy {
                damage: enemy_damage,
                material: enemy_material.clone(),
                max_turn_speed: std::f32::consts::PI * 0.5,
                move_force: 7.0,
                max_speed: 3.0,
                turn_torque: 20.0,
            },
            Damping {
                linear_damping: 0.8, // z.B. 0.5–2.0 testen
                angular_damping: 0.4,
            },
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(enemy_material),
            Transform::from_xyz(x, 0.5, z),
            Visibility::Visible,
            RigidBody::Dynamic,
            Collider::cuboid(0.5, 0.5, 0.5),
            ActiveEvents::COLLISION_EVENTS,
            EnemyShootTimer {
                timer: Timer::from_seconds(5.0, TimerMode::Repeating),
            },
            EnemyMovement {
                direction: Vec3::Z,
                change_timer: Timer::from_seconds(rng.random_range(1.0..3.0), TimerMode::Once),
            },
            ExternalImpulse::default(),
            ExternalForce::default(),
            Velocity::default(),
            (
                Friction {
                    coefficient: 0.1, // oder ein Wert nach Geschmack, z.B. 0.5–1.0
                    combine_rule: CoefficientCombineRule::Average,
                },
                ColliderMassProperties::Density(2.0),
            ),
        ));

        // Nach jedem Spawn neuen zufälligen Delay setzen (z.B. 0.5 bis 2.5 Sekunden)
        spawn_delay.timer = Timer::from_seconds(rng.random_range(0.5..2.5), TimerMode::Once);
    }
}

fn enemy_count_under_threshold(enemy_query: Query<(), With<Enemy>>) -> bool {
    enemy_query.iter().count() < 20
}

#[allow(clippy::too_many_arguments)]
fn enemy_shooting(
    mut commands: Commands,
    time: Res<Time>,
    mut enemy_query: Query<(&Transform, &mut ExternalImpulse, &mut EnemyShootTimer), With<Enemy>>,
    player_query: Single<&Transform, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemy_shoot_sound: Res<EnemyShootSound>, // <--- NEU
) {
    let bullet_speed = 30.0;
    let player_pos = player_query.translation;

    for (enemy_transform, mut impulse, mut shoot_timer) in enemy_query.iter_mut() {
        shoot_timer.timer.tick(time.delta());
        if !shoot_timer.timer.just_finished() {
            continue;
        }

        // Richtung zum Spieler
        let to_player = (player_pos - enemy_transform.translation).normalize();
        let shoot_direction = enemy_transform.forward();

        let recoil_strength = 5.0; // Stärke des Rückstoßes nach Geschmack
        let recoil_dir = -shoot_direction.normalize_or_zero();

        impulse.impulse += recoil_dir * recoil_strength;

        // Prüfe, ob Spieler im 90°-Sichtfeld vor dem Gegner ist
        if shoot_direction.dot(to_player) > 0.0 {
            // Beispiel im enemy_shooting-System (oder wo du die Bullet spawnst):
            let bullet_offset = 1.0; // Abstand vor dem Gegner (z.B. 1 Meter)
            let spawn_pos = enemy_transform.translation + shoot_direction * bullet_offset;

            commands.spawn((
                Bullet,
                BulletLifetime {
                    timer: Timer::from_seconds(4.0, TimerMode::Once),
                },
                Restitution {
                    coefficient: 1.0,
                    combine_rule: CoefficientCombineRule::Max,
                },
                Mesh3d(meshes.add(Sphere::new(0.2))),
                MeshMaterial3d(materials.add(Color::from(Srgba::new(1.0, 1.0, 0.0, 1.0)))),
                Transform::from_translation(spawn_pos),
                Visibility::Visible,
                RigidBody::Dynamic,
                Collider::ball(0.2),
                Velocity::linear(shoot_direction * bullet_speed),
                ActiveEvents::COLLISION_EVENTS,
                ColliderMassProperties::Density(2.0),
                Friction {
                    coefficient: 0.1, // oder ein Wert nach Geschmack, z.B. 0.5–1.0
                    combine_rule: CoefficientCombineRule::Average,
                },
                AudioPlayer::new(enemy_shoot_sound.0.clone()),
                PlaybackSettings::ONCE.with_spatial(true),
            ));
        }
    }
}

#[derive(Resource)]
struct EnemySpawnDelay {
    timer: Timer,
}

#[derive(Component)]
struct EnemyShootTimer {
    timer: Timer,
}

#[derive(Component)]
struct EnemyMovement {
    direction: Vec3,
    change_timer: Timer,
}
