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
                    delayed_death_system,
                    explosion_particle_system,
                    pending_explosion_animation_system,
                    maybe_spawn_enemy.run_if(enemy_count_under_threshold),
                )
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(
                Update,
                enemy_explosion_system.run_if(in_state(AppState::Running)),
            )
            .insert_resource(EnemySpawnDelay {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            });
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let explosion_sound = asset_server.load("sounds/explosion.wav");
    commands.insert_resource(ExplosionSound(explosion_sound));

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

#[allow(clippy::too_many_arguments)]
fn enemy_explosion_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut all_enemies: Query<(Entity, &Transform, Option<&mut ExternalImpulse>), With<Enemy>>,
    mut player_health: Single<&mut Health, With<Player>>,
    ground_entity: Single<Entity, With<Ground>>,
    pending_explosions: Query<Entity, With<PendingExplosion>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    explosion_sound: Res<ExplosionSound>, // <--- NEU
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let (enemy_entity, _) = if pending_explosions.get(*e1).is_ok() && *e2 == *ground_entity
            {
                (*e1, *e2)
            } else if pending_explosions.get(*e2).is_ok() && *e1 == *ground_entity {
                (*e2, *e1)
            } else {
                continue;
            };

            // Explosionszentrum bestimmen
            let explosion_pos = if let Ok((_, transform, _)) = all_enemies.get(enemy_entity) {
                transform.translation
            } else {
                continue;
            };

            // Partikel-Explosion erzeugen
            let num_particles = 10;
            let mut rng = rand::rng();
            for _ in 0..num_particles {
                let dir = Vec3::new(
                    rng.random_range(-1.0..1.0),
                    rng.random_range(0.0..1.0),
                    rng.random_range(-1.0..1.0),
                )
                .normalize_or_zero();
                let speed = rng.random_range(8.0..18.0);

                // Zufällige Variation für Start- und Endfarbe
                let start_r = (1.0 + rng.random_range(-0.05..0.05)).clamp(0.9, 1.0);
                let start_g = (0.2 + rng.random_range(-0.05..0.05)).clamp(0.1, 0.3);
                let start_b = rng.random_range(0.0..0.05);
                let end_r = (1.0 + rng.random_range(-0.05..0.05)).clamp(0.9, 1.0);
                let end_g = (0.8 + rng.random_range(-0.1..0.1)).clamp(0.6, 1.0);
                let end_b = rng.random_range(0.0..0.1);

                let start_color = Color::srgba(start_r, start_g, start_b, 1.0);
                let end_color = Color::srgba(end_r, end_g, end_b, 0.0);

                let mat_handle = materials.add(start_color);
                commands.spawn((
                    ExplosionParticle {
                        velocity: dir * speed,
                        timer: Timer::from_seconds(rng.random_range(0.3..0.7), TimerMode::Once),
                        start_color,
                        end_color,
                        material: mat_handle.clone(),
                    },
                    Mesh3d(meshes.add(Sphere::new(rng.random_range(0.08..0.18)))),
                    MeshMaterial3d(mat_handle),
                    Transform::from_translation(explosion_pos),
                    Visibility::Visible,
                ));
            }

            let mut killed = 0;
            for (other_entity, other_transform, impulse_opt) in all_enemies.iter_mut() {
                let dist = (other_transform.translation - explosion_pos).length();
                if dist <= 3.0 {
                    // Verzögertes Sterben
                    commands.entity(other_entity).insert(DelayedDeath {
                        timer: Timer::from_seconds(2.0, TimerMode::Once),
                    });
                    killed += 1;
                }
                if dist <= 10.0 {
                    // Impuls für alle im 10m-Radius (auch die im 1m-Kreis)
                    if let Some(mut impulse) = impulse_opt {
                        let dir = (other_transform.translation - explosion_pos)
                            .with_y(0.0)
                            .normalize_or_zero();
                        let strength = 20.0 * (1.0 - (dist / 10.0)).clamp(0.0, 1.0); // statt 30.0
                        impulse.impulse += dir * strength + Vec3::Y * (strength * 0.8);
                    }
                }
            }
            // Spieler Leben zurückgeben
            player_health.value += killed as f32 * 2.0;
            println!(
                "{} Gegner explodiert! Spieler bekommt {} Leben zurück.",
                killed,
                killed * 2
            );

            // Explodierten Gegner entfernen (falls nicht schon erledigt)
            commands.entity(enemy_entity).despawn();

            commands.spawn((
                AudioPlayer::new(explosion_sound.0.clone()),
                PlaybackSettings::ONCE
                    .with_spatial(true)
                    .with_volume(audio::Volume::Decibels(50.0)),
                Transform::from_translation(explosion_pos),
            ));
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

fn delayed_death_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DelayedDeath)>,
) {
    for (entity, mut death) in query.iter_mut() {
        death.timer.tick(time.delta());
        if death.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
struct ExplosionParticle {
    velocity: Vec3,
    timer: Timer,
    start_color: Color,
    end_color: Color,
    material: Handle<StandardMaterial>, // <--- NEU
}

fn explosion_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionParticle, &mut Transform)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut particle, mut transform) in query.iter_mut() {
        transform.translation += particle.velocity * time.delta_secs();
        particle.timer.tick(time.delta());

        let t = (particle.timer.elapsed_secs() / particle.timer.duration().as_secs_f32())
            .clamp(0.0, 1.0);
        let Srgba {
            red: start_r,
            green: start_g,
            blue: start_b,
            alpha: start_a,
        } = particle.start_color.to_srgba();
        let Srgba {
            red: end_r,
            green: end_g,
            blue: end_b,
            alpha: end_a,
        } = particle.end_color.to_srgba();
        let r = start_r + t * (end_r - start_r);
        let g = start_g + t * (end_g - start_g);
        let b = start_b + t * (end_b - start_b);
        let a = start_a + t * (end_a - start_a);
        let color = Color::srgba(r, g, b, a);

        if let Some(mat) = materials.get_mut(&particle.material) {
            mat.base_color = color;
        }

        if particle.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn pending_explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&Enemy, &mut PendingExplosion, &mut Transform, &mut Collider)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (enemy, mut anim, mut transform, mut collider) in query.iter_mut() {
        anim.timer.tick(time.delta());
        let t = (anim.timer.elapsed_secs() / anim.timer.duration().as_secs_f32()).clamp(0.0, 1.0);

        // Interpolierte Farbe
        let Srgba {
            red: sr,
            green: sg,
            blue: sb,
            alpha: sa,
        } = anim.start_color.to_srgba();
        let Srgba {
            red: er,
            green: eg,
            blue: eb,
            alpha: ea,
        } = anim.end_color.to_srgba();
        let color = Color::srgba(
            sr + t * (er - sr),
            sg + t * (eg - sg),
            sb + t * (eb - sb),
            sa + t * (ea - sa),
        );
        if let Some(mat) = materials.get_mut(&enemy.material) {
            mat.base_color = color;

            // Leuchteffekt: Emissive-Farbe interpoliert von 0 auf maximal
            let emissive_strength = t * 8.0; // Anfangs 0, am Ende maximal
            mat.emissive = LinearRgba::new(
                1.0 * emissive_strength,
                1.0 * emissive_strength,
                0.2 * emissive_strength,
                1.0,
            );
        }

        // Interpolierte Größe
        let scale = anim.start_scale.lerp(anim.end_scale, t);
        transform.scale = scale;

        // Collider anpassen (z.B. für einen Würfel)
        *collider = Collider::cuboid(0.5 * scale.x, 0.5 * scale.y, 0.5 * scale.z);

        // Nach Ablauf: NICHT despawnen, sondern ggf. Explosion auslösen!
        // (Das machst du weiterhin im enemy_explosion_system)
    }
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
            ));

            commands.spawn((
                AudioPlayer::new(enemy_shoot_sound.0.clone()),
                PlaybackSettings::ONCE.with_spatial(true),
                Transform::from_translation(spawn_pos),
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

#[derive(Component)]
pub struct PendingExplosion {
    timer: Timer,
    start_color: Color,
    end_color: Color,
    start_scale: Vec3,
    end_scale: Vec3,
}

impl PendingExplosion {
    pub fn new() -> Self {
        PendingExplosion {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            start_color: Color::from(Srgba::new(1.0, 0.2, 0.2, 1.0)),
            end_color: Color::from(Srgba::new(1.0, 1.0, 0.2, 1.0)),
            start_scale: Vec3::ONE,
            end_scale: Vec3::splat(2.0),
        }
    }
}

#[derive(Component)]
struct DelayedDeath {
    timer: Timer,
}

#[derive(Resource, Clone)]
struct ExplosionSound(Handle<AudioSource>);
