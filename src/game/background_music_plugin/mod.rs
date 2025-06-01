use bevy::prelude::*;
use bevy::time::Timer;
use bevy_procedural_audio::prelude::*;
use fundsp::hacker32::{sine, split, square, triangle, var};
use std::sync::Arc;
use uuid::Uuid;

mod music;

pub struct BackgroundMusicPlugin;

impl Plugin for BackgroundMusicPlugin {
    fn build(&self, app: &mut App) {
        let melody_freqs = Arc::new(
            music::melody_pattern()
                .iter()
                .filter_map(|&m| m) // Filter out None values
                .map(|&m| midi_to_freq(m))
                .collect::<Vec<_>>(),
        );
        let bass_freqs = Arc::new(
            music::bass_pattern()
                .iter()
                .filter_map(|&m| m) // Filter out None values
                .map(|&m| midi_to_freq(m))
                .collect::<Vec<_>>(),
        );
        let chord_freqs = Arc::new(
            music::chord_pattern()
                .iter()
                .filter_map(|&m| m) // Filter out None values
                .map(|&m| midi_to_freq(m))
                .collect::<Vec<_>>(),
        );

        let idx = shared(0.0);

        let melody_freq = shared(melody_freqs[0]);
        let bass_freq = shared(bass_freqs[0]);
        let chord_freq = shared(chord_freqs[0]);

        let music_graph = {
            let melody_freq2 = melody_freq.clone();
            let bass_freq2 = bass_freq.clone();
            let chord_freq2 = chord_freq.clone();
            move || {
                var(&melody_freq2)
                    >> sine() * 0.2 + var(&bass_freq2)
                    >> square() * 0.12 + var(&chord_freq2)
                    >> triangle() * 0.08
                    >> split::<U2>()
            }
        };

        let music_dsp = MyMusicDsp(music_graph);
        let music_id = music_dsp.id();

        app.add_plugins(DspPlugin::default())
            .add_dsp_source(music_dsp, SourceType::Dynamic)
            .insert_resource(MyMusicIdx(idx))
            .insert_resource(MyMusicFreqs(melody_freqs))
            .insert_resource(MyBassFreqs(bass_freqs))
            .insert_resource(MyChordFreqs(chord_freqs))
            .insert_resource(MyMusicFreq(melody_freq))
            .insert_resource(MyBassFreq(bass_freq))
            .insert_resource(MyChordFreq(chord_freq))
            .insert_resource(MyMusicId(music_id))
            .insert_resource(MyPatternLen(music::melody_pattern().len()))
            .add_systems(PostStartup, play_music)
            .add_systems(Update, advance_pattern);
    }
}

// Structs für Shared-Variablen
#[derive(Resource)]
pub struct MyMusicIdx(pub Shared);
#[derive(Resource)]
pub struct MyMusicFreqs(pub Arc<Vec<f32>>);
#[derive(Resource)]
pub struct MyBassFreqs(pub Arc<Vec<f32>>);
#[derive(Resource)]
pub struct MyChordFreqs(pub Arc<Vec<f32>>);
#[derive(Resource)]
pub struct MyMusicFreq(pub Shared);
#[derive(Resource)]
pub struct MyBassFreq(pub Shared);
#[derive(Resource)]
pub struct MyChordFreq(pub Shared);
#[derive(Resource)]
pub struct MyMusicId(pub Uuid);
#[derive(Resource)]
pub struct MyPatternLen(pub usize);

// DSP-Graph-Wrapper wie im Piano-Beispiel
pub struct MyMusicDsp<F>(pub F);

impl<T: AudioUnit + 'static, F: Send + Sync + 'static + Fn() -> T> DspGraph for MyMusicDsp<F> {
    fn id(&self) -> Uuid {
        Uuid::from_u128(0x123456789abcdef0123456789abcdef0u128)
    }
    fn generate_graph(&self) -> Box<dyn AudioUnit> {
        Box::new((self.0)())
    }
}

// Musik abspielen (wird nach Startup aufgerufen)
fn play_music(
    mut commands: Commands,
    mut assets: ResMut<Assets<DspSource>>,
    dsp_manager: Res<DspManager>,
    music_id: Res<MyMusicId>,
) {
    let source = assets.add(
        dsp_manager
            .get_graph_by_id(&music_id.0)
            .unwrap_or_else(|| panic!("DSP source not found!")),
    );
    commands.spawn(AudioPlayer(source));
}

// System: Pattern-Index regelmäßig weiterschalten (z.B. alle 0.4 Sekunden)
fn advance_pattern(
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
    idx: Res<MyMusicIdx>,
    melody_freqs: Res<MyMusicFreqs>,
    bass_freqs: Res<MyBassFreqs>,
    chord_freqs: Res<MyChordFreqs>,
    melody_freq: Res<MyMusicFreq>,
    bass_freq: Res<MyBassFreq>,
    chord_freq: Res<MyChordFreq>,
    pattern_len: Res<MyPatternLen>,
) {
    // Timer initialisieren (z.B. alle 0.4 Sekunden)
    if timer.is_none() {
        *timer = Some(Timer::from_seconds(0.4, TimerMode::Repeating));
    }
    let timer = timer.as_mut().unwrap();

    if timer.tick(time.delta()).just_finished() {
        let mut i = idx.0.value();
        i = (i + 1.0) % pattern_len.0 as f32;
        idx.0.set_value(i);
        let idx_usize = i as usize;
        melody_freq.0.set_value(melody_freqs.0[idx_usize]);
        bass_freq.0.set_value(bass_freqs.0[idx_usize]);
        chord_freq.0.set_value(chord_freqs.0[idx_usize]);
    }
}

fn midi_to_freq(midi: i32) -> f32 {
    440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0)
}
