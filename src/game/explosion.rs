use bevy::{audio, prelude::*};
use bevy_rapier3d::{na::RealField, prelude::*};
use rand::{Rng, seq::IndexedRandom};

use crate::{AppState, Ground};

use super::{
    bullet::Bullet,
    enemy::Enemy,
    health::Health,
    player::Player,
    tree::{RootParticleAssets, TreeCollider, TreeRoot, maybe_uproot_tree},
};

fn pending_explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut PendingExplosion)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut anim) in query.iter_mut() {
        anim.timer.tick(time.delta());
        let t = (anim.timer.elapsed_secs() / anim.timer.duration().as_secs_f32()).clamp(0.0, 1.0);

        // TODO: Make that generic
        // Interpolierte Farbe (von start_color zu end_color)
        // let Srgba {
        //     red: sr,
        //     green: sg,
        //     blue: sb,
        //     alpha: sa,
        // } = anim.start_color.to_srgba();
        // let Srgba {
        //     red: er,
        //     green: eg,
        //     blue: eb,
        //     alpha: ea,
        // } = anim.end_color.to_srgba();
        // let color = Color::srgba(
        //     sr + t * (er - sr),
        //     sg + t * (eg - sg),
        //     sb + t * (eb - sb),
        //     sa + t * (ea - sa),
        // );

        // if let Some(mat) = materials.get_mut(material_handle) {
        //     mat.base_color = color;

        //     // Emissive-Farbe interpolieren
        //     let emissive_strength = t * anim.emissive_strength;
        //     mat.emissive = LinearRgba::new(
        //         1.0 * emissive_strength,
        //         1.0 * emissive_strength,
        //         0.2 * emissive_strength,
        //         1.0,
        //     );
        // }
    }
}

#[derive(Component)]
pub struct PendingExplosion {
    pub timer: Timer,
    pub lighten_factor: f32,
    pub emissive_strength: f32,
    pub start_scale: Vec3,
    pub end_scale: Vec3,
    pub dead_zone_radius: f32,
    pub affected_zone_radius: f32,
}

impl PendingExplosion {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        timer_secs: f32,
        lighten_factor: f32,
        emissive_strength: f32,
        start_scale: Vec3,
        end_scale: Vec3,
        dead_zone_radius: f32,
        affected_zone_radius: f32,
    ) -> Self {
        PendingExplosion {
            timer: Timer::from_seconds(timer_secs, TimerMode::Once),
            lighten_factor,
            emissive_strength,
            start_scale,
            end_scale,
            dead_zone_radius,
            affected_zone_radius,
        }
    }
}

impl Default for PendingExplosion {
    fn default() -> Self {
        PendingExplosion {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            lighten_factor: 1.0,
            emissive_strength: 8.0,
            start_scale: Vec3::ONE,
            end_scale: Vec3::splat(2.0),
            dead_zone_radius: 3.0,
            affected_zone_radius: 10.0,
        }
    }
}

#[derive(Resource, Clone)]
struct ExplosionSound(Handle<AudioSource>);

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
#[allow(clippy::too_many_arguments)]
fn ground_explosion_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    pending_explosions: Query<(&Transform, Entity), With<PendingExplosion>>,
    children_query: Query<&ChildOf, Without<PendingExplosion>>,
    mut all_enemies: Query<(Entity, &Transform, Option<&mut ExternalImpulse>), With<Enemy>>,
    mut player_health: Single<&mut Health, With<Player>>,
    ground_entity: Single<Entity, With<Ground>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    explosion_sound: Res<ExplosionSound>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let other = if *e1 == *ground_entity {
                e2
            } else if *e2 == *ground_entity {
                e1
            } else {
                continue;
            };
            let Some((explosion_transform, pending_entity)) =
                pending_explosions.get(*other).ok().or_else(|| {
                    children_query
                        .get(*other)
                        .ok()
                        .and_then(|ChildOf(p)| pending_explosions.get(*p).ok())
                })
            else {
                continue;
            };

            explode_pending_entity(
                &mut commands,
                pending_entity,
                explosion_transform.translation,
                &mut all_enemies,
                &mut player_health,
                &mut meshes,
                &mut materials,
                &explosion_sound,
            );
        }
    }
}

fn explode_pending_entity(
    commands: &mut Commands,
    entity: Entity,
    explosion_pos: Vec3,
    all_enemies: &mut Query<(Entity, &Transform, Option<&mut ExternalImpulse>), With<Enemy>>,
    player_health: &mut Single<&mut Health, With<Player>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    explosion_sound: &Res<ExplosionSound>,
) {
    // Partikel-Explosion erzeugen (wie gehabt)
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

    // Schaden und Impuls für Gegner im Umkreis
    let mut killed = 0;
    for (other_entity, other_transform, impulse_opt) in all_enemies.iter_mut() {
        let dist = (other_transform.translation - explosion_pos).length();
        if dist <= 3.0 {
            commands.entity(other_entity).insert(DelayedDeath {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
            });
            killed += 1;
        }
        if dist <= 10.0 {
            if let Some(mut impulse) = impulse_opt {
                let dir = (other_transform.translation - explosion_pos)
                    .with_y(0.0)
                    .normalize_or_zero();
                let strength = 20.0 * (1.0 - (dist / 10.0)).clamp(0.0, 1.0);
                impulse.impulse += dir * strength + Vec3::Y * (strength * 0.8);
            }
        }
    }
    player_health.value += killed as f32 * 2.0;
    println!(
        "{} Gegner explodiert! Spieler bekommt {} Leben zurück.",
        killed,
        killed * 2
    );

    // Entity entfernen
    commands.entity(entity).despawn();

    // Sound abspielen
    commands.spawn((
        AudioPlayer::new(explosion_sound.0.clone()),
        PlaybackSettings::ONCE
            .with_spatial(true)
            .with_volume(audio::Volume::Decibels(50.0)),
        Transform::from_translation(explosion_pos),
    ));
}

pub fn bullet_collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut player_query: Query<(Entity, &mut Health), With<Player>>,
    mut enemy_query: Query<(Entity, &mut ExternalImpulse), With<Enemy>>,
    bullet_query: Query<(Entity, &Velocity), With<Bullet>>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // Finde die Bullet und ihre Velocity
            let ((bullet_entity, bullet_velocity), other) = if let Ok(b) = bullet_query.get(*e1) {
                (b, e2)
            } else if let Ok(b) = bullet_query.get(*e2) {
                (b, e1)
            } else {
                continue;
            };

            // Prüfe, ob der andere ein Spieler ist
            if let Ok((player_entity, mut health)) = player_query.get_mut(*other) {
                health.value -= 1.0;
                println!("Spieler getroffen! Leben: {}", health.value);
                continue;
            }

            // Prüfe, ob der andere ein Gegner ist
            if let Ok((enemy_entity, mut impulse)) = enemy_query.get_mut(*other) {
                let dir = bullet_velocity.linvel.normalize_or_zero();
                let impulse_vec = dir * 60.0 + Vec3::Y * 6.0;
                impulse.impulse += impulse_vec;

                // Explosion vormerken
                commands
                    .entity(enemy_entity)
                    .insert(PendingExplosion::default());

                commands.entity(bullet_entity).despawn();
                continue;
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let explosion_sound = asset_server.load("sounds/explosion.wav");
    commands.insert_resource(ExplosionSound(explosion_sound));
}

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                tree_explosion_system,
                explosion_particle_system,
                player_tree_collision_system,
                pending_explosion_animation_system,
                ground_explosion_system,
                delayed_death_system,
                bullet_collision_system,
            )
                .run_if(in_state(AppState::Running)),
        );
    }
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
struct DelayedDeath {
    timer: Timer,
}

fn tree_explosion_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    pending_explosions: Query<Entity, With<PendingExplosion>>,
    tree_query: Query<(Entity, &Children, &Transform), (With<TreeRoot>, Without<PendingExplosion>)>,
    tree_colliders_query: Query<
        (Entity, &ChildOf),
        (With<TreeCollider>, Without<PendingExplosion>),
    >,
    mut all_enemies: Query<(Entity, &Transform, Option<&mut ExternalImpulse>), With<Enemy>>,
    mut player_health: Single<&mut Health, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    explosion_sound: Res<ExplosionSound>,
    root_assets: Res<RootParticleAssets>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // Prüfe, ob eine Seite PendingExplosion und die andere ein Baum ohne PendingExplosion ist
            let (pending_entity, tree_child_entity) = if pending_explosions.contains(*e1) {
                let Ok((_, ChildOf(parent))) = tree_colliders_query.get(*e2) else {
                    continue;
                };
                (*e1, *parent)
            } else if pending_explosions.contains(*e2) {
                let Ok((_, ChildOf(parent))) = tree_colliders_query.get(*e1) else {
                    continue;
                };
                (*e2, *parent)
            } else {
                continue;
            };

            maybe_uproot_tree(
                &mut commands,
                tree_child_entity,
                &tree_query,
                &tree_colliders_query,
                &root_assets,
            );

            // --- pending_entity explodieren lassen wie im explosion_system ---
            if let Ok((_, transform, _)) = all_enemies.get(pending_entity) {
                explode_pending_entity(
                    &mut commands,
                    pending_entity,
                    transform.translation,
                    &mut all_enemies,
                    &mut player_health,
                    &mut meshes,
                    &mut materials,
                    &explosion_sound,
                );
            }
        }
    }
}

fn player_tree_collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    player: Single<(Entity, &Velocity, &Player)>,
    tree_query: Query<(Entity, &Children, &Transform), (With<TreeRoot>, Without<PendingExplosion>)>,
    tree_colliders_query: Query<
        (Entity, &ChildOf),
        (With<TreeCollider>, Without<PendingExplosion>),
    >,
    root_assets: Res<RootParticleAssets>,
) {
    let (player_entity, player_velocity, Player { speed, .. }) = player.into_inner();

    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // Prüfe, ob Player und Baum beteiligt sind
            let tree_entity = if player_entity == *e1 {
                let Ok((_, ChildOf(parent))) = tree_colliders_query.get(*e2) else {
                    continue;
                };
                *parent
            } else if player_entity == *e2 {
                let Ok((_, ChildOf(parent))) = tree_colliders_query.get(*e1) else {
                    continue;
                };
                *parent
            } else {
                continue;
            };

            // Prüfe Geschwindigkeit
            let current_speed = player_velocity.linvel.length();
            if current_speed < speed - 10. {
                continue;
            }

            maybe_uproot_tree(
                &mut commands,
                tree_entity,
                &tree_query,
                &tree_colliders_query,
                &root_assets,
            );

            // Optional: Sound, Partikel, etc. wie bei tree_explosion_system
        }
    }
}
