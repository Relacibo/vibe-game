use bevy::math::primitives::*;
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
use bevy_rapier3d::prelude::*;
use game::explosion::ExplosionPlugin;
use game::tree::TreePlugin;
use noise::{NoiseFn, Perlin};
pub mod assets;
pub mod game;

use bevy::window::{CursorGrabMode, PrimaryWindow};
use game::{
    background_music_plugin::BackgroundMusicPlugin,
    bullet::BulletPlugin,
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
    .add_plugins(ExplosionPlugin)
    .add_plugins(TreePlugin)
    .add_systems(Startup, setup.after(setup_skybox)) // <--- Reihenfolge explizit!
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

    #[cfg(debug_assertions)]
    app.add_plugins(RapierDebugRenderPlugin::default());

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    skybox_handle: Res<SkyboxHandle>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
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
    let mud_texture = asset_server.load_with_settings("textures/mud_ground.png", |s: &mut _| {
        *s = ImageLoaderSettings {
            sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..default()
            }),
            ..default()
        }
    });
    let mud_normal =
        asset_server.load_with_settings("textures/mud_ground_normal.png", |s: &mut _| {
            *s = ImageLoaderSettings {
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..default()
                }),
                ..default()
            }
        });
    let mud_gloss =
        asset_server.load_with_settings("textures/mud_ground_gloss.png", |s: &mut _| {
            *s = ImageLoaderSettings {
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..default()
                }),
                ..default()
            }
        });

    let mud_material = materials.add(StandardMaterial {
        base_color_texture: Some(mud_texture),
        normal_map_texture: Some(mud_normal),
        metallic_roughness_texture: Some(mud_gloss), // Glossmap als Roughness-Map
        perceptual_roughness: 0.7,
        reflectance: 0.05,
        ..default()
    });

    let mesh_handle = meshes.add(Mesh::from(Plane3d {
        normal: Dir3::Y,
        half_size: Vec2::splat(ground_size / 2.0),
    }));

    // UVs anpassen für Kachelung
    if let Some(mesh) = meshes.get_mut(&mesh_handle) {
        if let Some(VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            for uv in uvs.iter_mut() {
                uv[0] *= ground_size / 32.0; // 32.0 = sehr oft gekachelt!
                uv[1] *= ground_size / 32.0;
            }
        }
    }

    commands.spawn((
        Ground,
        Mesh3d(mesh_handle),
        MeshMaterial3d(mud_material),
        Transform::from_xyz(0.0, -0.05, 0.0),
        Visibility::Visible,
        RigidBody::Fixed,
        Collider::cuboid(ground_size / 2.0, 0.05, ground_size / 2.0),
        Friction {
            coefficient: 0.1,
            combine_rule: CoefficientCombineRule::Average,
        },
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::DYNAMIC_STATIC,
    ));

    // Wände (jeweils 2 km lang, 10 m hoch, 1 m dick)
    let wall_color = Color::srgb(0.8, 0.8, 0.8);
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
}

// Walls-Hilfsfunktion (wie gehabt)
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
