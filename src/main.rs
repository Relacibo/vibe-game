use bevy::{
    audio::{self, AudioPlayer, PlaybackSettings},
    core_pipeline::Skybox,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    prelude::*,
    render::{
        mesh::VertexAttributeValues,
        render_resource::{TextureViewDescriptor, TextureViewDimension},
        renderer::RenderDevice,
    },
};
use bevy_rapier3d::{na::RealField, prelude::*};
use std::f32::consts::PI;

pub mod assets;
pub mod game;

use bevy::ecs::system::ParamSet;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use game::player::{Player, player_movement_system};
use game::skybox_plugin::{SkyboxHandle, SkyboxPlugin};
use game::{
    enemy::{Bullet, Enemy},
    skybox_plugin::setup_skybox,
};
use game::{gui::GuiPlugin, health::Health};
use rand::Rng;
use rand::rng;

#[derive(Component)]
struct BulletLifetime {
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
struct PendingExplosion {
    timer: Timer,
    start_color: Color,
    end_color: Color,
    start_scale: Vec3,
    end_scale: Vec3,
}

#[derive(Component)]
struct Ground;

#[derive(Resource)]
struct EnemySpawnDelay {
    timer: Timer,
}

#[derive(Component)]
struct DelayedDeath {
    timer: Timer,
}

#[derive(Resource, Clone)]
struct BounceSound(Handle<AudioSource>);

#[derive(Resource, Clone)]
struct ExplosionSound(Handle<AudioSource>);

#[derive(Resource, Clone)]
struct EnemyShootSound(Handle<AudioSource>);

// --- In deiner main() ---
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // provide the ID selector string here
                #[cfg(target_family = "wasm")]
                canvas: Some("#bevy-canvas".into()),
                // ... any other window properties ...
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(SkyboxPlugin)
        .add_plugins(GuiPlugin)
        .add_systems(Startup, setup.after(setup_skybox)) // <--- Reihenfolge explizit!
        .add_systems(
            Update,
            (
                player_movement_system,
                camera_follow_system,
                enemy_movement_system,
                enemy_shooting,
                bullet_player_collision_system,
                bullet_enemy_collision_system, // <--- NEU
                enemy_explosion_system,        // <--- NEU
                bullet_lifetime_system,
                delayed_death_system,
                bounce_sound_system,
                enemy_despawn_far_system,
                explosion_particle_system,
                pending_explosion_animation_system,
                maybe_spawn_enemy.run_if(enemy_count_under_threshold),
            ),
        )
        .insert_resource(EnemySpawnDelay {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        })
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    skybox_handle: Res<SkyboxHandle>,
    asset_server: Res<AssetServer>,
) {
    // Maus einfangen und verstecken
    window.cursor_options.grab_mode = CursorGrabMode::Confined;
    window.cursor_options.visible = false;

    // Kamera mit Skybox
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Skybox {
            image: skybox_handle.0.clone(),
            brightness: 1000.0,
            ..default()
        },
    ));

    let mut rng = rng();

    // Player (blauer Würfel)
    let player_speed = rng.random_range(1.0..=5.0);
    commands.spawn((
        Player {
            speed: player_speed,
        },
        Health { value: 100.0 },
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::from(Srgba::new(0.2, 0.2, 1.0, 1.0)))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Visibility::Visible,
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 0.5),
        Velocity::zero(),
        ExternalForce::default(), // <-- Diese Zeile ist beim Player noch da, das ist korrekt!
        ExternalImpulse::default(),
        Friction {
            coefficient: 0.5, // oder ein Wert nach Geschmack, z.B. 0.5–1.0
            combine_rule: CoefficientCombineRule::Average,
        },
        Restitution::default(),
        AdditionalMassProperties::Mass(1.0), // Setze die Masse auf 1.0 (Standard ist oft viel höher)
        // Optional: Trägheit auf sehr klein setzen, damit Rotation leicht geht
        // AdditionalMassProperties::MassProperties(MassProperties {
        //     local_center_of_mass: Vec3::ZERO,
        //     mass: 1.0,
        //     principal_inertia: Vec3::splat(0.01),
        //     ..default()
        // }),
        ActiveEvents::COLLISION_EVENTS,
    ));

    // Boden (2 km x 2 km)
    let ground_size = 2000.0;
    let tiles_texture = asset_server.load_with_settings("textures/tiles.png", |s: &mut _| {
        *s = ImageLoaderSettings {
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..default()
            }),
            ..default()
        }
    });
    let tiles_material = materials.add(StandardMaterial {
        base_color_texture: Some(tiles_texture),
        perceptual_roughness: 0.9,
        reflectance: 0.1,
        ..default()
    });

    // Plane3d-Mesh mit richtiger Größe erzeugen
    let plane_mesh = Plane3d::default()
        .mesh()
        .size(ground_size, ground_size)
        .subdivisions(10);
    let mesh_handle = meshes.add(plane_mesh);

    // UVs anpassen (Kacheln)
    if let Some(mesh) = meshes.get_mut(&mesh_handle) {
        if let Some(VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            for uv in uvs.iter_mut() {
                uv[0] *= ground_size / 100.0; // z.B. 1000 Kacheln in X
                uv[1] *= ground_size / 100.0; // z.B. 1000 Kacheln in Y
            }
        }
    }

    commands.spawn((
        Ground, // <--- Tag-Komponente
        Mesh3d(mesh_handle),
        MeshMaterial3d(tiles_material),
        Transform::from_xyz(0.0, -0.05, 0.0),
        Visibility::Visible,
        RigidBody::Fixed,
        Collider::cuboid(ground_size / 2.0, 0.05, ground_size / 2.0),
        Friction {
            coefficient: 0.5, // oder ein Wert nach Geschmack, z.B. 0.5–1.0
            combine_rule: CoefficientCombineRule::Average,
        },
    ));

    // Wände (jeweils 2 km lang, 10 m hoch, 1 m dick)
    let wall_color = Color::from(Srgba::new(0.8, 0.8, 0.8, 1.0));
    let wall_material = materials.add(wall_color);
    spawn_wall(
        &mut commands,
        &mut meshes,
        wall_material.clone(),
        Vec3::new(2000.0, 10.0, 1.0),
        Vec3::new(0.0, 5.0, 1000.0),
    ); // Nord
    spawn_wall(
        &mut commands,
        &mut meshes,
        wall_material.clone(),
        Vec3::new(2000.0, 10.0, 1.0),
        Vec3::new(0.0, 5.0, -1000.0),
    ); // Süd
    spawn_wall(
        &mut commands,
        &mut meshes,
        wall_material.clone(),
        Vec3::new(1.0, 10.0, 2000.0),
        Vec3::new(1000.0, 5.0, 0.0),
    ); // Ost
    spawn_wall(
        &mut commands,
        &mut meshes,
        wall_material.clone(),
        Vec3::new(1.0, 10.0, 2000.0),
        Vec3::new(-1000.0, 5.0, 0.0),
    ); // West

    let bounce_sound = asset_server.load("sounds/bounce.wav");
    commands.insert_resource(BounceSound(bounce_sound));

    let explosion_sound = asset_server.load("sounds/explosion.wav");
    commands.insert_resource(ExplosionSound(explosion_sound));

    let enemy_shoot_sound = asset_server.load("sounds/arrow_shoot.wav");
    commands.insert_resource(EnemyShootSound(enemy_shoot_sound));
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
            ));

            commands.spawn((
                AudioPlayer::new(enemy_shoot_sound.0.clone()),
                PlaybackSettings::ONCE.with_spatial(true),
                Transform::from_translation(spawn_pos),
            ));
        }
    }
}

#[allow(clippy::type_complexity)]
fn camera_follow_system(
    mut param_set: ParamSet<(
        Single<&Transform, With<Player>>,
        Single<&mut Transform, With<Camera3d>>,
    )>,
) {
    let player_translation = param_set.p0().translation;
    let player_forward = param_set.p0().forward();

    let mut camera_transform = param_set.p1();

    // Offset hinter dem Spieler (z.B. 5 Einheiten zurück und 3 Einheiten nach oben)
    let back_offset = -player_forward * 5.0 + Vec3::Y * 3.0;

    camera_transform.translation = player_translation + back_offset;
    camera_transform.look_at(player_translation + player_forward * 10.0, Vec3::Y);
}

fn bullet_player_collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_query: Query<(Entity, &mut Health), With<Player>>,
    bullet_query: Query<Entity, With<Bullet>>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let (player_entity, mut health) = if let Ok(p) = player_query.get_mut(*e1) {
                (e1, p.1)
            } else if let Ok(p) = player_query.get_mut(*e2) {
                (e2, p.1)
            } else {
                continue;
            };

            let bullet_entity = if bullet_query.get(*e1).is_ok() {
                *e1
            } else if bullet_query.get(*e2).is_ok() {
                *e2
            } else {
                continue;
            };

            // Spieler bekommt Schaden
            health.value -= 1.0;
            // Bullet entfernen
            // commands.entity(bullet_entity).despawn();
            println!("Spieler getroffen! Leben: {}", health.value);
        }
    }
}

fn bullet_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BulletLifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn enemy_movement_system(
    time: Res<Time>,
    player_query: Single<&Transform, With<Player>>,
    mut enemy_query: Query<
        (
            &Transform,
            &Velocity,
            &mut EnemyMovement,
            &mut ExternalForce,
        ),
        With<Enemy>,
    >,
) {
    let player_pos = player_query.translation;
    let max_turn_speed = std::f32::consts::PI * 0.5; // 90°/s
    let move_force = 6.0;
    let max_speed = 3.0;
    let turn_torque = 30.0; // Stärke des Drehmoments

    for (enemy_transform, velocity, mut movement, mut force) in enemy_query.iter_mut() {
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
            let max_angle = max_turn_speed * time.delta_secs();
            let clamped_angle = angle.min(max_angle);

            if clamped_angle > 0.001 && axis.length_squared() > 0.0 {
                let sign = axis.y.signum();
                force.torque = Vec3::Y * sign * turn_torque * clamped_angle;
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

        // Zerlege die aktuelle Kraft in Bewegungsrichtung und Rest
        let force_in_dir = force.force.dot(world_dir);
        let force_rest = force.force - world_dir * force_in_dir;

        // Nur wenn Geschwindigkeit < max_speed, setze die Kraft in Bewegungsrichtung auf move_force
        if vel_in_dir < max_speed {
            force.force = force_rest + world_dir * move_force;
        } else {
            // Wenn zu schnell, entferne die Kraft in Bewegungsrichtung, Rest bleibt erhalten
            force.force = force_rest;
        }
    }
}

fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    size: Vec3,
    position: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(position),
        Visibility::Visible,
        RigidBody::Fixed,
        Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
    ));
}

fn bullet_enemy_collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut enemy_query: Query<(Entity, &mut ExternalImpulse), With<Enemy>>,
    bullet_query: Query<(Entity, &Velocity), With<Bullet>>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let (bullet_entity, bullet_velocity) = if let Ok(b) = bullet_query.get(*e1) {
                b
            } else if let Ok(b) = bullet_query.get(*e2) {
                b
            } else {
                continue;
            };

            let (enemy_entity, mut impulse) = if let Ok(e) = enemy_query.get_mut(*e1) {
                e
            } else if let Ok(e) = enemy_query.get_mut(*e2) {
                e
            } else {
                continue;
            };

            let dir = bullet_velocity.linvel.normalize_or_zero();
            let impulse_vec = dir * 60.0 + Vec3::Y * 6.0;
            impulse.impulse += impulse_vec;

            // Explosion vormerken
            commands.entity(enemy_entity).insert(PendingExplosion {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
                start_color: Color::from(Srgba::new(1.0, 0.2, 0.2, 1.0)), // Ursprungsfarbe
                end_color: Color::from(Srgba::new(1.0, 1.0, 0.2, 1.0)),   // z.B. gelblich
                start_scale: Vec3::ONE,
                end_scale: Vec3::splat(2.0),
            });

            // Kugel entfernen
            commands.entity(bullet_entity).despawn();
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
                health: enemy_health,
                damage: enemy_damage,
                material: enemy_material.clone(), // <--- NEU
            },
            Damping {
                linear_damping: 1.0, // z.B. 0.5–2.0 testen
                angular_damping: 1.0,
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
            Friction {
                coefficient: 0.5, // oder ein Wert nach Geschmack, z.B. 0.5–1.0
                combine_rule: CoefficientCombineRule::Average,
            },
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

fn bounce_sound_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    bounce_sound: Res<BounceSound>,
    bullet_query: Query<(&Transform, Entity), With<Bullet>>,
    player_transform: Single<&Transform, With<Player>>,
) {
    let max_distance_from_player = 2.0;
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // Prüfe, ob einer der beiden ein Bullet ist und wie weit entfernt
            if let Ok((bullet_transform, _)) = bullet_query.get(*e1) {
                if (bullet_transform.translation - player_transform.translation).length()
                    < max_distance_from_player
                {
                    commands.spawn((
                        AudioPlayer::new(bounce_sound.0.clone()),
                        PlaybackSettings::ONCE.with_spatial(true),
                    ));
                }
            } else if let Ok((bullet_transform, _)) = bullet_query.get(*e2) {
                if (bullet_transform.translation - player_transform.translation).length()
                    < max_distance_from_player
                {
                    commands.spawn((
                        AudioPlayer::new(bounce_sound.0.clone()),
                        PlaybackSettings::ONCE.with_spatial(true),
                    ));
                }
            }
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
    mut query: Query<(&Enemy, &mut PendingExplosion, &mut Transform)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (enemy, mut anim, mut transform) in query.iter_mut() {
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
        transform.scale = anim.start_scale.lerp(anim.end_scale, t);

        // Nach Ablauf: NICHT despawnen, sondern ggf. Explosion auslösen!
        // (Das machst du weiterhin im enemy_explosion_system)
    }
}
