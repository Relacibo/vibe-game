use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_rapier3d::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{Rng, seq::IndexedRandom};
use serde::Deserialize;

use crate::AppState;

use super::{explosion::PendingExplosion, player::Player};

#[derive(Resource)]
pub struct RootParticleAssets {
    pub roots: Vec<Handle<Scene>>,
    pub splitters: Vec<Handle<Scene>>,
}

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<TreeColliderInfo>::new(&[
            "tree_collider.json",
        ]))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            update_tree_colliders.run_if(in_state(AppState::Running)),
        )
        .add_systems(Update, despawn_root_particles);
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            // Prüfe Mindestabstand zu allen bisherigen Bäumen
            if tree_positions.iter().all(|&(px, pz)| {
                let dx = px - x;
                let dz = pz - z;
                (dx * dx + dz * dz).sqrt() >= min_distance
            }) {
                let idx = rng.random_range(0..trees.trees.len());
                let tree = &trees.trees[idx];

                let y_rot = rng.random_range(0.0..std::f32::consts::TAU);

                commands.spawn((
                    SceneRoot(tree.scene_handle.clone()),
                    Transform {
                        translation: Vec3::new(x, 0.0, z),
                        rotation: Quat::from_rotation_y(y_rot),
                        scale: Vec3::splat(3.0),
                    },
                    Visibility::Visible,
                    RigidBody::Fixed,
                    TreeRoot { idx },
                ));

                tree_positions.push((x, z));
                spawned += 1;
            }
        }
    }
}

#[derive(Component)]
pub struct TreeRoot {
    idx: usize, // Index im Trees-Array
}

#[derive(Component)]
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
            .map(|c| c.iter().any(|child| collider_query.get(child).is_ok()))
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
                ));
                parent.spawn((
                    Collider::ball(crown.radius),
                    Transform::from_xyz(crown.center[0], crown.center[1], crown.center[2]),
                    TreeCollider,
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

pub fn maybe_uproot_tree(
    commands: &mut Commands,
    tree_entity: Entity,
    tree_query: &Query<
        (Entity, &Children, &Transform),
        (With<TreeRoot>, Without<PendingExplosion>),
    >,
    tree_colliders_query: &Query<
        (Entity, &ChildOf),
        (With<TreeCollider>, Without<PendingExplosion>),
    >,
    root_assets: &Res<RootParticleAssets>,
) {
    // PendingExplosion setzen
    commands.entity(tree_entity).insert(PendingExplosion {
        timer: Timer::from_seconds(2.0, TimerMode::Once),
        lighten_factor: 1.0,
        emissive_strength: 8.0,
        start_scale: Vec3::ONE,
        end_scale: Vec3::splat(2.0),
        dead_zone_radius: 6.,
        affected_zone_radius: 15.0,
    });

    // Collider-Kinder entfernen
    let Ok((_, children, tree_transform)) = tree_query.get(tree_entity) else {
        return;
    };
    for child in children.iter() {
        let Ok((child, _)) = tree_colliders_query.get(child) else {
            continue;
        };
        commands
            .entity(child)
            .remove::<RigidBody>()
            .insert(ColliderMassProperties::Density(2.0));
    }

    // Baum dynamisch machen und Impuls geben
    use bevy_rapier3d::prelude::*;
    let mut rng = rand::rng();
    let impulse = Vec3::new(
        rng.random_range(-2.0..2.0),
        rng.random_range(12.0..18.0), // nach oben
        rng.random_range(-2.0..2.0),
    );

    commands.entity(tree_entity).insert((
        RigidBody::Dynamic,
        AdditionalMassProperties::Mass(5.),
        ExternalImpulse {
            impulse,
            torque_impulse: Vec3::new(
                rng.random_range(-8.0..8.0),
                rng.random_range(-8.0..8.0),
                rng.random_range(-8.0..8.0),
            ),
        },
    ));

    let mut rng = rand::rng();
    // for _ in 0..3 {
    //     let root_scene = root_assets.roots.choose(&mut rng).unwrap().clone();
    //     let angle = rng.random_range(0.0..std::f32::consts::TAU);
    //     let dir = Vec3::new(angle.cos(), rng.random_range(0.5..1.2), angle.sin()).normalize();
    //     let impulse = dir * rng.random_range(8.0..16.0);

    //     commands.spawn((
    //         SceneRoot(root_scene),
    //         Transform::from_translation(tree_transform.translation)
    //             .with_rotation(Quat::from_rotation_y(angle)),
    //         Visibility::Visible,
    //         RootParticle {
    //             timer: Timer::from_seconds(3.0, TimerMode::Once),
    //         },
    //         ExternalImpulse {
    //             impulse,
    //             torque_impulse: Vec3::new(
    //                 rng.random_range(-2.0..2.0),
    //                 rng.random_range(-2.0..2.0),
    //                 rng.random_range(-2.0..2.0),
    //             ),
    //         },
    //         RigidBody::Dynamic,
    //         Collider::cylinder(0.4, 0.08), // grober Collider für Physik
    //     ));
    // }
    // for _ in 0..2 {
    //     let splitter_scene = root_assets.splitters.choose(&mut rng).unwrap().clone();
    //     let angle = rng.random_range(0.0..std::f32::consts::TAU);
    //     let dir = Vec3::new(angle.cos(), rng.random_range(0.7..1.5), angle.sin()).normalize();
    //     let impulse = dir * rng.random_range(1.0..2.0);

    //     commands.spawn((
    //         SceneRoot(splitter_scene),
    //         Transform::from_translation(tree_transform.translation)
    //             .with_rotation(Quat::from_rotation_y(angle)),
    //         Visibility::Visible,
    //         RootParticle {
    //             timer: Timer::from_seconds(3.0, TimerMode::Once),
    //         },
    //         ExternalImpulse {
    //             impulse,
    //             torque_impulse: Vec3::new(
    //                 rng.random_range(-3.0..3.0),
    //                 rng.random_range(-3.0..3.0),
    //                 rng.random_range(-3.0..3.0),
    //             ),
    //         },
    //         ColliderMassProperties::Density(2.0),
    //         RigidBody::Dynamic,
    //         Collider::cylinder(0.2, 0.04),
    //     ));
    // }
}
