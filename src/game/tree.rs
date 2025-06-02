use bevy::audio::{self, AudioPlayer, PlaybackSettings};
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_rapier3d::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{Rng, seq::IndexedRandom};
use serde::Deserialize;

use crate::AppState;
use crate::game::explosion::PendingExplosionSuppressed;

use super::{explosion::PendingExplosion, player::Player};

#[derive(Resource)]
pub struct RootParticleAssets {
    pub roots: Vec<Handle<Scene>>,
    pub splitters: Vec<Handle<Scene>>,
}

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(AssetLoadingState::default())
            .add_plugins(JsonAssetPlugin::<TreeColliderInfo>::new(&[
                "tree_collider.json",
            ]))
            .add_systems(Startup, pre_setup)
            .add_systems(OnEnter(AssetLoadingState::Done), setup)
            .add_systems(
                Update,
                (
                    update_tree_colliders,
                    despawn_root_particles,
                    animate_particles,
                )
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(Update, check_assets_loaded);
    }
}

#[derive(Resource, Deserialize, Debug, Clone, bevy::asset::Asset, bevy::reflect::TypePath)]
struct TreeColliderInfo {
    trunk: ColliderPart,
    crown: ColliderPart,
}

#[derive(Debug, Deserialize, Clone)]
struct ColliderPart {
    center: [f32; 3],
    radius: f32,
    height: f32,
}

#[derive(Debug, Clone)]
struct Tree {
    scene_handle: Handle<Scene>,
    collider_info: Handle<TreeColliderInfo>,
}

#[derive(Clone, Resource)]
struct Trees {
    trees: [Tree; 12],
}

impl Trees {
    fn new(asset_server: &Res<AssetServer>) -> Self {
        let trees: [Tree; 12] = std::array::from_fn(|i| {
            let scene_handle = asset_server.load(format!("models/trees/tree_{i}.glb#Scene0"));
            let collider_info =
                asset_server.load(format!("models/trees/tree_{i}.tree_collider.json"));
            Tree {
                scene_handle,
                collider_info,
            }
        });
        Self { trees }
    }
}

#[derive(Resource)]
pub struct StakeSound(pub Handle<AudioSource>);

fn pre_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Trees-Resource mit Collider-Infos laden
    let trees = Trees::new(&asset_server);
    commands.insert_resource(trees.clone());
    // RootParticleAssets laden
    let mut roots = Vec::new();
    let mut splitters = Vec::new();
    for i in 0..5 {
        roots.push(asset_server.load(format!("models/roots/root_{i}.glb#Scene0")));
    }
    for i in 0..4 {
        splitters.push(asset_server.load(format!("models/roots/root_splitter_{i}.glb#Scene0")));
    }
    commands.insert_resource(RootParticleAssets { roots, splitters });
    // Stake-Sound laden
    let stake_sound = asset_server.load("sounds/stake.wav");
    commands.insert_resource(StakeSound(stake_sound));
}

fn setup(mut commands: Commands, collider_infos: Res<Assets<TreeColliderInfo>>, trees: Res<Trees>) {
    // Bäume platzieren
    let perlin = Perlin::new(42);
    let mut rng = rand::rng();

    let ground_size = 2000.0;
    let tree_count = 800;
    let min_distance = 6.0; // Mindestabstand zwischen Bäumen
    let mut spawned = 0;
    let mut tries = 0;
    let mut tree_positions: Vec<(f32, f32)> = Vec::with_capacity(tree_count);

    while spawned < tree_count && tries < tree_count * 10 {
        tries += 1;
        let x = rng.random_range(-ground_size / 2.0..ground_size / 2.0);
        let z = rng.random_range(-ground_size / 2.0..ground_size / 2.0);

        // Für größere Wäldchen: Noise-Schwelle niedriger und Skalierung kleiner
        let noise_val = perlin.get([x as f64 / 800.0, z as f64 / 800.0]);
        let is_wood = noise_val > 0.18;
        let is_lonely = rng.random_bool(0.01);

        if is_wood || is_lonely {
            let y_rot = rng.random_range(0.0..std::f32::consts::TAU);
            // Prüfe Mindestabstand zu allen bisherigen Bäumen
            if tree_positions.iter().all(|&(px, pz)| {
                let dx = px - x;
                let dz = pz - z;
                (dx * dx + dz * dz).sqrt() >= min_distance
            }) {
                let idx = rng.random_range(0..trees.trees.len());
                let tree = &trees.trees[idx];
                let Some(collider_info) = collider_infos.get(&tree.collider_info) else {
                    continue;
                };

                let TreeColliderInfo {
                    trunk:
                        ColliderPart {
                            radius: trunk_radius,
                            ..
                        },
                    ..
                } = *collider_info;

                commands.spawn((
                    SceneRoot(tree.scene_handle.clone()),
                    Transform {
                        translation: Vec3::new(x, 0.0, z),
                        rotation: Quat::from_rotation_y(y_rot),
                        scale: Vec3::splat(3.0),
                    },
                    Visibility::Visible,
                    RigidBody::Fixed,
                    TreeRoot { idx, trunk_radius },
                ));

                tree_positions.push((x, z));
                spawned += 1;
            }
        }
    }
}

#[derive(Debug, Component)]
pub struct TreeRoot {
    idx: usize,        // Index im Trees-Array
    trunk_radius: f32, // Stammbreite
}

#[derive(Debug, Component)]
pub struct TreeCollider;

fn update_tree_colliders(
    mut commands: Commands,
    player_query: Single<&Transform, With<Player>>,
    tree_query: Query<(Entity, &Transform, &TreeRoot, Option<&Children>)>,
    collider_query: Query<&TreeCollider>,
    collider_infos: Res<Assets<TreeColliderInfo>>,
    trees: Res<Trees>,
) {
    let player_pos = player_query.translation;
    let cull_distance = 80.0; // z.B. 80 Meter

    for (entity, tree_transform, tree_root, children) in tree_query.iter() {
        let tree_pos = tree_transform.translation;
        let dist = player_pos.distance(tree_pos);

        let has_collider = children
            .map(|c| c.iter().any(|child| collider_query.contains(child)))
            .unwrap_or(false);

        if dist < cull_distance && !has_collider {
            // Collider SPAWNEN
            let tree = &trees.trees[tree_root.idx];
            let Some(collider_info) = collider_infos.get(&tree.collider_info) else {
                warn!("Collider not present! {}", tree_root.idx);
                return;
            };

            let TreeColliderInfo { trunk, crown } = collider_info;

            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Collider::cylinder(trunk.height / 2.0, trunk.radius),
                    Transform::from_xyz(trunk.center[0], trunk.center[1], trunk.center[2]),
                    TreeCollider,
                    ColliderMassProperties::Density(2.0),
                ));
                parent.spawn((
                    Collider::ball(crown.radius),
                    Transform::from_xyz(crown.center[0], crown.center[1], crown.center[2]),
                    TreeCollider,
                    ColliderMassProperties::Density(2.0),
                ));
            });
        } else if dist >= cull_distance && has_collider {
            // Collider ENTFERNEN
            if let Some(children) = children {
                for &child in children {
                    if collider_query.get(child).is_ok() {
                        commands.entity(child).despawn();
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct RootParticle {
    timer: Timer,
    velocity: Vec3,
    rotation_speed: Option<Vec3>, // Nur für Splitter gesetzt
}

fn despawn_root_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut RootParticle)>,
) {
    for (entity, mut particle) in query.iter_mut() {
        particle.timer.tick(time.delta());
        if particle.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn maybe_uproot_tree(
    commands: &mut Commands,
    tree_entity: Entity,
    tree_query: &Query<(Entity, &Children, &Transform, &TreeRoot), Without<PendingExplosion>>,
    root_assets: &Res<RootParticleAssets>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    stake_sound: &Res<StakeSound>,
) {
    // Collider-Kinder entfernen
    let Ok((_, children, tree_transform, tree_root)) = tree_query.get(tree_entity) else {
        return;
    };

    for child_entity in children {
        commands
            .entity(*child_entity)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(ActiveCollisionTypes::DYNAMIC_STATIC);
    }

    // Baum dynamisch machen und Impuls geben
    let mut rng = rand::rng();
    let impulse = Vec3::new(
        rng.random_range(-2.0..2.0),
        400.0, // nach oben
        rng.random_range(-2.0..2.0),
    );

    commands.entity(tree_entity).insert((
        PendingExplosion {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            lighten_factor: 1.0,
            emissive_strength: 8.0,
            start_scale: Vec3::ONE,
            end_scale: Vec3::splat(2.0),
            dead_zone_radius: 6.,
            affected_zone_radius: 15.0,
        },
        PendingExplosionSuppressed::default(),
        RigidBody::Dynamic,
        ExternalImpulse {
            impulse,
            torque_impulse: Vec3::new(
                rng.random_range(-8.0..8.0),
                rng.random_range(-8.0..8.0),
                rng.random_range(-8.0..8.0),
            ),
        },
        AudioPlayer::new(stake_sound.0.clone()),
        PlaybackSettings::ONCE.with_spatial(true),
    ));

    // let max_root_speed = impulse.length(); // Baum-Impuls als Maximum
    let mut rng = rand::rng();
    let spread = tree_root.trunk_radius * 1.2;

    // Wurzeln (animiert)
    for _ in 0..6 {
        let root_scene = root_assets.roots.choose(&mut rng).unwrap().clone();
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let offset = Vec3::new(
            rng.random_range(-spread..spread),
            -1.2, // weiter unten spawnen!
            rng.random_range(-spread..spread),
        );
        let spawn_pos = tree_transform.translation + offset;
        let dir = Vec3::new(angle.cos() * 0.45, 0.5, angle.sin() * 0.45).normalize(); // mehr nach außen, weniger nach oben
        let velocity = dir * rng.random_range(7.0..11.0); // etwas schneller

        // Rotation um die Flugrichtung (vom Baum weg)
        let axis = dir.normalize();
        let angle_speed = rng.random_range(-3.0..3.0);
        let rotation_speed = axis * angle_speed;

        commands.spawn((
            SceneRoot(root_scene),
            Transform::from_translation(spawn_pos),
            Visibility::Visible,
            RootParticle {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
                velocity,
                rotation_speed: Some(rotation_speed),
            },
        ));
    }

    // Splitter (animiert)
    for _ in 0..12 {
        let splitter_scene = root_assets.splitters.choose(&mut rng).unwrap().clone();
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let offset = Vec3::new(
            rng.random_range(-spread..spread),
            0.0,
            rng.random_range(-spread..spread),
        );
        let spawn_pos = tree_transform.translation + offset;
        let dir = Vec3::new(
            angle.cos() * 0.7,
            rng.random_range(0.1..0.4),
            angle.sin() * 0.7,
        )
        .normalize();
        let velocity = dir * rng.random_range(10.0..16.0);
        let rotation_speed = Vec3::new(
            rng.random_range(-6.0..6.0),
            rng.random_range(-6.0..6.0),
            rng.random_range(-6.0..6.0),
        );
        commands.spawn((
            SceneRoot(splitter_scene),
            Transform::from_translation(spawn_pos).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                rng.random_range(0.0..std::f32::consts::TAU),
                rng.random_range(0.0..std::f32::consts::TAU),
                rng.random_range(0.0..std::f32::consts::TAU),
            )),
            Visibility::Visible,
            RootParticle {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
                velocity,
                rotation_speed: Some(rotation_speed),
            },
        ));
    }

    // Dreck (animiert, viele, größer, variabel, seitlich)
    let dirt_color = Color::srgb(0.25, 0.15, 0.07);
    for _ in 0..160 {
        let offset = Vec3::new(
            rng.random_range(-spread..spread),
            0.0,
            rng.random_range(-spread..spread),
        );
        let spawn_pos = tree_transform.translation + offset;
        let dirt_dir = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(0.05..0.25),
            rng.random_range(-1.0..1.0),
        )
        .normalize();
        let velocity = dirt_dir * rng.random_range(18.0..32.0);
        let scale = rng.random_range(0.2..0.5); // 10x größer

        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(bevy::math::primitives::Sphere { radius: 0.08 }))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: dirt_color,
                perceptual_roughness: 1.0,
                ..default()
            })),
            Transform {
                translation: spawn_pos,
                scale: Vec3::splat(scale),
                ..Default::default()
            },
            Visibility::Visible,
            RootParticle {
                timer: Timer::from_seconds(1.5, TimerMode::Once),
                velocity,
                rotation_speed: None,
            },
        ));
    }
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum AssetLoadingState {
    #[default]
    Loading,
    Done,
}

fn check_assets_loaded(
    mut next_state: ResMut<NextState<AssetLoadingState>>,
    trees: Option<Res<Trees>>,
    collider_infos: Option<Res<Assets<TreeColliderInfo>>>,
    asset_server: Res<AssetServer>,
) {
    let Some(trees) = trees else {
        return;
    };
    let Some(collider_infos) = collider_infos else {
        return;
    };

    let all_tree_collider_loaded = trees.trees.iter().all(|tree| {
        asset_server.is_loaded(&tree.collider_info)
            && collider_infos.get(&tree.collider_info).is_some()
    });

    if all_tree_collider_loaded {
        next_state.set(AssetLoadingState::Done);
    }
}

// Animationssystem für die Partikel
fn animate_particles(time: Res<Time>, mut query: Query<(&mut Transform, &mut RootParticle)>) {
    let dt = time.delta_secs();
    for (mut transform, mut particle) in query.iter_mut() {
        // Einfache Schwerkraft
        particle.velocity.y -= 9.81 * dt;
        transform.translation += particle.velocity * dt;

        // Splitter-Rotation animieren
        if let Some(rot_speed) = particle.rotation_speed {
            let delta_rot = Quat::from_euler(
                EulerRot::XYZ,
                rot_speed.x * dt,
                rot_speed.y * dt,
                rot_speed.z * dt,
            );
            transform.rotation = delta_rot * transform.rotation;
        }
    }
}
