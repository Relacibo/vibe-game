use crate::{AppState, Enemy, Health, Player};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct Bullet;

#[derive(Resource, Clone)]
struct BounceSound(Handle<AudioSource>);

#[derive(Component)]
pub struct BulletLifetime {
    pub timer: Timer,
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bounce_sound = asset_server.load("sounds/bounce.wav");
    commands.insert_resource(BounceSound(bounce_sound));
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                bullet_lifetime_system,
                bounce_sound_system,
            )
                .run_if(in_state(AppState::Running)),
        );
    }
}
