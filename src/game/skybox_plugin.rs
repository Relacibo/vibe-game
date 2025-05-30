use bevy::{
    core_pipeline::Skybox,
    image::CompressedImageFormats,
    prelude::*,
    render::{
        render_resource::{TextureViewDescriptor, TextureViewDimension},
        renderer::RenderDevice,
    },
};
use std::f32::consts::PI;

const CUBEMAPS: &[(&str, CompressedImageFormats)] = &[
    ("textures/Ryfjallet_cubemap.png", CompressedImageFormats::NONE),
    ("textures/Ryfjallet_cubemap_astc4x4.ktx2", CompressedImageFormats::ASTC_LDR),
    ("textures/Ryfjallet_cubemap_bc7.ktx2", CompressedImageFormats::BC),
    ("textures/Ryfjallet_cubemap_etc2.ktx2", CompressedImageFormats::ETC2),
];

#[derive(Resource, Clone)]
pub struct SkyboxHandle(pub Handle<Image>);

pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_skybox)
           .add_systems(Update, (
                cycle_cubemap_asset,
                asset_loaded.after(cycle_cubemap_asset),
                animate_light_direction,
            ));
    }
}

pub fn setup_skybox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
) {
    // Directional 'sun' light
    commands.spawn((
        DirectionalLight {
            illuminance: 7000.0, // statt 32000.0
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 0.0).with_rotation(Quat::from_rotation_x(-PI / 4.)),
    ));

    // CubeMap-Format wählen, das von der Hardware unterstützt wird
    let supported_compressed_formats =
        CompressedImageFormats::from_features(render_device.features());
    let mut cubemap_path = CUBEMAPS[0].0;
    for (path, format) in CUBEMAPS {
        if supported_compressed_formats.contains(*format) {
            cubemap_path = path;
            break;
        }
    }

    let skybox_handle = asset_server.load(cubemap_path);

    // Skybox-Handle als Resource speichern
    commands.insert_resource(SkyboxHandle(skybox_handle.clone()));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb_u8(210, 220, 240),
        brightness:  40.0,
        ..default()
    });

    // Für Cubemap-Wechsel
    commands.insert_resource(Cubemap {
        is_loaded: false,
        index: 0,
        image_handle: skybox_handle,
    });
}

#[derive(Resource)]
struct Cubemap {
    is_loaded: bool,
    index: usize,
    image_handle: Handle<Image>,
}

const CUBEMAP_SWAP_DELAY: f32 = 3.0;

fn cycle_cubemap_asset(
    time: Res<Time>,
    mut next_swap: Local<f32>,
    mut cubemap: ResMut<Cubemap>,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
) {
    let now = time.elapsed_secs();
    if *next_swap == 0.0 {
        *next_swap = now + CUBEMAP_SWAP_DELAY;
        return;
    } else if now < *next_swap {
        return;
    }
    *next_swap += CUBEMAP_SWAP_DELAY;

    let supported_compressed_formats =
        CompressedImageFormats::from_features(render_device.features());

    let mut new_index = cubemap.index;
    for _ in 0..CUBEMAPS.len() {
        new_index = (new_index + 1) % CUBEMAPS.len();
        if supported_compressed_formats.contains(CUBEMAPS[new_index].1) {
            break;
        }
        info!(
            "Skipping format which is not supported by current hardware: {:?}",
            CUBEMAPS[new_index]
        );
    }

    if new_index == cubemap.index {
        return;
    }

    cubemap.index = new_index;
    cubemap.image_handle = asset_server.load(CUBEMAPS[cubemap.index].0);
    cubemap.is_loaded = false;
}

fn asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_loaded && asset_server.load_state(&cubemap.image_handle).is_loaded() {
        info!("Swapping to {}...", CUBEMAPS[cubemap.index].0);
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(image.height() / image.width());
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.image = cubemap.image_handle.clone();
        }

        cubemap.is_loaded = true;
    }
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 0.5);
    }
}
