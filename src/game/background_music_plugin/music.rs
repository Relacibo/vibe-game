use rand::seq::IndexedRandom;
use std::sync::Arc;

pub const C_MAJOR: [i32; 8] = [60, 62, 64, 65, 67, 69, 71, 72];

pub const CHORDS: &[(&str, &[i32])] = &[
    ("I", &[60, 64, 67]),
    ("ii", &[62, 65, 69]),
    ("iii", &[64, 67, 71]),
    ("IV", &[65, 69, 72]),
    ("V", &[67, 71, 74]),
    ("V7", &[67, 71, 74, 77]),
    ("vi", &[69, 72, 76]),
    ("bVII", &[70, 74, 77]),
];

pub const PROGRESSION: [&str; 16] = [
    "I", "vi", "ii", "V7", "iii", "vi", "IV", "bVII", "I", "V7", "vi", "IV", "ii", "V", "I", "V7",
];

pub const RHYTHM_PATTERNS: &[&[f32]] = &[
    &[0.25, 0.25, 0.5, 0.5],
    &[0.375, 0.125, 0.375, 0.125, 0.5],
    &[0.333, 0.167, 0.333, 0.167, 0.333],
    &[0.5, 0.25, 0.25, 0.5],
    &[0.375, 0.125, 0.25, 0.375],
    &[0.25, 0.25, 0.25, 0.25, 0.5],
    &[0.125, 0.125, 0.125, 0.125, 0.25, 0.25],
];

pub const ALLOWED_INTERVALS: [i32; 8] = [0, 2, 4, 5, 7, 9, 10, 11];

pub fn allowed_note(chord: &[i32], note: i32) -> bool {
    for &c in chord {
        let interval = ((note - c).rem_euclid(12)).abs();
        if ALLOWED_INTERVALS.contains(&interval) {
            // Vermeide Terz+Quarte gleichzeitig
            if !(interval == 3 || interval == 4) || !ALLOWED_INTERVALS.contains(&5) {
                return true;
            }
        }
    }
    false
}

pub fn pick_note(chord: &[i32], scale: &[i32], prev_note: i32) -> i32 {
    let mut candidates: Vec<i32> = scale
        .iter()
        .cloned()
        .filter(|&n| allowed_note(chord, n) && (n - prev_note).abs() <= 7)
        .collect();
    if candidates.is_empty() {
        candidates = scale
            .iter()
            .cloned()
            .filter(|&n| allowed_note(chord, n))
            .collect();
    }
    *candidates.choose(&mut rand::rng()).unwrap_or(&prev_note)
}

pub fn make_bass_for_chord(chord: &[i32], rhythm: &[f32]) -> Vec<Option<i32>> {
    let root = chord[0] - 12;
    let fifth = chord[2] - 12;
    let pattern = [root, fifth];
    rhythm
        .iter()
        .enumerate()
        .map(|(i, _)| {
            if rand::random::<f32>() < 0.15 {
                Some(root + 12)
            } else {
                Some(pattern[i % 2])
            }
        })
        .collect()
}

pub fn make_voice_for_chord(
    chord: &[i32],
    scale: &[i32],
    prev_note: i32,
    rhythm: &[f32],
    is_melody: bool,
    other_voice_rhythm: Option<&[Option<i32>]>,
) -> (Vec<Option<i32>>, i32) {
    let mut voice = Vec::new();
    let mut last_note = prev_note;
    for (i, _) in rhythm.iter().enumerate() {
        if let Some(other) = other_voice_rhythm {
            if i < other.len() && other[i].is_some() && rand::random::<f32>() < 0.5 {
                voice.push(None);
                continue;
            }
        }
        if rand::random::<f32>() < if is_melody { 0.15 } else { 0.25 } {
            voice.push(None);
            continue;
        }
        let note = pick_note(chord, scale, last_note);
        voice.push(Some(note));
        last_note = note;
    }
    (voice, last_note)
}

pub fn melody_pattern() -> Vec<Option<i32>> {
    vec![
        Some(64),
        None,
        Some(66),
        Some(68),
        None,
        Some(69),
        Some(71),
        None,
        Some(69),
        Some(68),
        None,
        Some(66),
    ]
}
pub fn bass_pattern() -> Vec<Option<i32>> {
    vec![
        Some(40),
        Some(40),
        None,
        Some(43),
        Some(43),
        None,
        Some(45),
        Some(45),
        None,
        Some(43),
        Some(43),
        None,
    ]
}
pub fn chord_pattern() -> Vec<Option<i32>> {
    vec![
        Some(52),
        None,
        Some(55),
        Some(59),
        None,
        Some(52),
        Some(55),
        None,
        Some(59),
        Some(52),
        None,
        Some(55),
    ]
}
pub fn rhythm_pattern() -> Vec<f32> {
    vec![0.25, 0.5, 0.25, 0.5]
}

pub fn midi_to_freq(midi: i32) -> f32 {
    440.0 * 2.0f32.powf((midi as f32 - 69.0) / 12.0)
}

pub fn get_frequencies() -> (Arc<Vec<Option<f32>>>, Arc<Vec<Option<f32>>>, Arc<Vec<Option<f32>>>) {
    let melody_freqs = Arc::new(
        music::melody_pattern()
            .iter()
            .map(|&m| m.map(midi_to_freq))
            .collect::<Vec<_>>(),
    );
    let bass_freqs = Arc::new(
        music::bass_pattern()
            .iter()
            .map(|&m| m.map(midi_to_freq))
            .collect::<Vec<_>>(),
    );
    let chord_freqs = Arc::new(
        music::chord_pattern()
            .iter()
            .map(|&m| m.map(midi_to_freq))
            .collect::<Vec<_>>(),
    );
    (melody_freqs, bass_freqs, chord_freqs)
}
