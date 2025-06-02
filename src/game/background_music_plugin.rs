use bevy::audio::{AudioPlayer, PlaybackSettings, Volume};
use bevy::prelude::*;

pub struct BackgroundMusicPlugin;

impl Plugin for BackgroundMusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, play_background_music);
    }
}

fn play_background_music(asset_server: Res<AssetServer>, mut commands: Commands) {
    // commands.spawn((
    //     AudioPlayer::new(asset_server.load("music/vibe_8bit_theme.wav")),
    //     PlaybackSettings {
    //         mode: bevy::audio::PlaybackMode::Loop,
    //         volume: Volume::Linear(0.2), // oder z.B. Volume::Decibels(-6.0)
    //         ..Default::default()
    //     },
    // ));
}
