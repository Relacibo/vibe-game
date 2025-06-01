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
use noise::{NoiseFn, Perlin};
use rand::seq::SliceRandom;
use std::f32::consts::PI;

pub mod assets;
pub mod game;

use bevy::ecs::system::ParamSet;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;
use game::{
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
use rand::rng;

#[derive(Component)]
struct Ground;

#[derive(States, PartialEq, Eq, Clone, Copy, Debug, Hash, Default)]
pub enum AppState {
    #[default]
    Running,
    Paused,
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
    .add_plugins(SkyboxPlugin)
    .add_plugins(GuiPlugin)
    .add_plugins(BulletPlugin)
    .add_plugins(EnemyPlugin)
    .add_plugins(CameraPlugin)
    .add_plugins(PlayerPlugin)
    .add_systems(
        Startup,
        (setup.after(setup_skybox), spawn_trees.after(setup)),
    ) // <--- Reihenfolge explizit!
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

    app.add_plugins(PauseMenuPlugin);

    app.run();
}

#[derive(Resource)]
struct Trees {
    scene_handles: [Handle<Scene>; 12],
}

impl Trees {
    fn new(mesh_handles: [Handle<Scene>; 12]) -> Self {
        Self {
            scene_handles: mesh_handles,
        }
    }
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

    // Bäume-GLB-Scenes laden
    let mut mesh_handles: [Handle<Scene>; 12] = Default::default();
    #[allow(clippy::needless_range_loop)]
    for i in 0..mesh_handles.len() {
        mesh_handles[i] = asset_server
            .load(format!("models/trees/tree_{i}.glb#Scene0"));
    }
    commands.insert_resource(Trees::new(mesh_handles));
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
    let tree_count = 3000;
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
                let idx = rng.random_range(0..12);

                commands.spawn((
                    SceneRoot(trees.scene_handles[idx].clone()),
                    Transform::from_xyz(x, 0.0, z),
                    Visibility::Visible,
                ));

                tree_positions.push((x, z));
                spawned += 1;
            }
        }
    }
}
