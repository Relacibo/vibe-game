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
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_rapier3d::{na::RealField, prelude::*};
use noise::{NoiseFn, Perlin};
use rand::seq::SliceRandom;
use std::{f32::consts::PI, mem::zeroed};

pub mod assets;
pub mod game;

use bevy::ecs::system::ParamSet;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use game::{
    background_music_plugin::BackgroundMusicPlugin,
    bullet::{Bullet, BulletLifetime, BulletPlugin, bullet_collision_system},
    camera::CameraPlugin,
    enemy::EnemyPlugin,
    player::PlayerPlugin,
    skybox_plugin::{SkyboxHandle, SkyboxPlugin},
};
use game::{enemy::Enemy, skybox_plugin::setup_skybox};
use game::{gui::GuiPlugin, health::Health};
use game::{pause_menu_gui::PauseMenuPlugin, player::Player};
use rand::Rng;
use serde::Deserialize;

#[derive(Component)]
struct Ground;

#[derive(States, PartialEq, Eq, Clone, Copy, Debug, Hash, Default)]
pub enum AppState {
    #[default]
    Running,
    Paused,
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

#[derive(Resource)]
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

// --- In deiner main() ---
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
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
    .add_plugins(BackgroundMusicPlugin)
    .add_plugins(SkyboxPlugin)
    .add_plugins(GuiPlugin)
    .add_plugins(BulletPlugin)
    .add_plugins(EnemyPlugin)
    .add_plugins(CameraPlugin)
    .add_plugins(PlayerPlugin)
    .add_plugins(PauseMenuPlugin)
    .add_plugins(JsonAssetPlugin::<TreeColliderInfo>::new(&[
        "tree_collider.json",
    ]))
    .add_systems(
        Startup,
        (setup.after(setup_skybox), spawn_trees.after(setup)),
    ) // <--- Reihenfolge explizit!
    .add_systems(
        Update,
        update_tree_colliders.run_if(in_state(AppState::Running)),
    )
    .configure_sets(
        Update,
        (
            PhysicsSet::SyncBackend,
            PhysicsSet::StepSimulation,
            PhysicsSet::Writeback,
        )
            .run_if(in_state(AppState::Running)),
    );

    #[cfg(target_family = "wasm")]
    app.insert_state(AppState::Paused);

    #[cfg(not(target_family = "wasm"))]
    app.insert_state(AppState::default());

    app.run();
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
    #[cfg(not(target_family = "wasm"))]
    {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }

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
            coefficient: 0.1, // oder ein Wert nach Geschmack, z.B. 0.5–1.0
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

    // Trees-Resource mit Collider-Infos laden
    commands.insert_resource(Trees::new(&asset_server));
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

fn spawn_trees(mut commands: Commands, trees: Res<Trees>) {
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
struct TreeRoot {
    idx: usize, // Index im Trees-Array
}

#[derive(Component)]
struct TreeCollider;

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
                warn!("Collider für Entity {:?} nicht gefunden!", entity);
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
