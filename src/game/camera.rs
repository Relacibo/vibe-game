use bevy::{
    input::{
        common_conditions::input_pressed,
        mouse::{AccumulatedMouseScroll, MouseMotion, MouseScrollUnit},
    },
    prelude::*,
};

use crate::AppState;

const CAMERA_ZOOM_SPEED: f32 = 0.2;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraControl>().add_systems(
            Update,
            (
                camera_input_system,
                camera_zoom_system,
                camera_follow_system
                    .after(camera_input_system)
                    .after(camera_zoom_system),
            )
                .run_if(in_state(AppState::Running)),
        );
    }
}

#[derive(Resource)]
pub struct CameraControl {
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
}

impl Default for CameraControl {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            zoom: 5.0,
        }
    }
}

fn camera_input_system(
    mut camera_control: ResMut<CameraControl>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    let mut delta = Vec2::ZERO;
    for ev in mouse_motion_events.read() {
        delta += ev.delta;
    }
    camera_control.yaw -= delta.x * 0.002;
    camera_control.pitch = (camera_control.pitch - delta.y * 0.002).clamp(-0.3, 1.4);
}

fn camera_follow_system(
    camera_control: Res<CameraControl>,
    player_transform: Single<&Transform, With<crate::Player>>,
    mut camera_transform: Single<&mut Transform, (With<Camera3d>, Without<crate::Player>)>,
    time: Res<Time>,
) {
    let player_translation = player_transform.translation;
    let look_at_offset = Vec3::Y * (0.7 + 0.25 * camera_control.zoom);
    let look_at = player_translation + look_at_offset;

    // Richtung von LookAt zur Kamera (aus Yaw und Pitch)
    let dir = Vec3::new(
        camera_control.yaw.sin() * camera_control.pitch.cos(),
        camera_control.pitch.sin(),
        camera_control.yaw.cos() * camera_control.pitch.cos(),
    )
    .normalize();

    let target_pos = look_at + dir * camera_control.zoom;

    camera_transform.translation = camera_transform
        .translation
        .lerp(target_pos, 1.0 - (-8.0 * time.delta_secs()).exp());
    camera_transform.look_at(look_at, Vec3::Y);
}

fn camera_zoom_system(
    mut camera_control: ResMut<CameraControl>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    if mouse_wheel_input.delta.y == 0.0 {
        return;
    }

    let delta_y = if mouse_wheel_input.unit == MouseScrollUnit::Line {
        -mouse_wheel_input.delta.y
    } else {
        -mouse_wheel_input.delta.y / 100.0
    };

    camera_control.zoom *= 1.0 + delta_y * CAMERA_ZOOM_SPEED;
    camera_control.zoom = camera_control.zoom.clamp(2.0, 30.0);
}
