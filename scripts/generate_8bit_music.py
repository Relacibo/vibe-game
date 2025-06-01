import mido
from mido import Message, MidiFile, MidiTrack
import random
import os

script_dir = os.path.dirname(os.path.abspath(__file__))
output_dir = os.path.join(script_dir, "..", "generated", "generate_8bit_music")
os.makedirs(output_dir, exist_ok=True)
output_path = os.path.join(output_dir, "vibe_8bit_theme.mid")

C_MAJOR = [60, 62, 64, 65, 67, 69, 71, 72]
CHORDS = {
    "I":  [60, 64, 67],
    "ii": [62, 65, 69],
    "iii": [64, 67, 71],
    "IV": [65, 69, 72],
    "V":  [67, 71, 74],
    "V7": [67, 71, 74, 77],
    "vi": [69, 72, 76],
    "bVII": [70, 74, 77],
}

PROGRESSION = [
    "I", "vi", "ii", "V7",
    "iii", "vi", "IV", "bVII",
    "I", "V7", "vi", "IV",
    "ii", "V", "I", "V7"
] * 2

RHYTHM_PATTERNS = [
    [240, 240, 480, 480],
    [360, 120, 360, 120, 480],
    [320, 160, 320, 160, 320],
    [480, 240, 240, 480],
    [360, 120, 240, 360],
    [240, 240, 240, 240, 480],
    [120, 120, 120, 120, 240, 240],
]

# Erlaubte Intervalle (in Halbtonschritten) zu Akkordtönen
ALLOWED_INTERVALS = [0, 2, 4, 5, 7, 9, 10, 11]  # Prime, große Sekunde, große/kleine Terz, Quarte, Quinte, große/kleine Sexte, große/kleine Septime

def allowed_note(chord, note):
    # Erlaubt, wenn zu irgendeinem Akkordton ein erlaubtes Intervall besteht
    for c in chord:
        interval = abs((note - c) % 12)
        if interval in ALLOWED_INTERVALS:
            # Vermeide Terz+Quarte gleichzeitig (z.B. E+F zu C-Dur)
            if not (interval == 3 or interval == 4) or not (abs((note - c - 5) % 12) == 5):
                return True
    return False

def pick_note(chord, scale, prev_note):
    # Suche einen erlaubten Ton in der Nähe
    candidates = [n for n in scale if allowed_note(chord, n) and abs(n - prev_note) <= 7]
    if not candidates:
        candidates = [n for n in scale if allowed_note(chord, n)]
    return random.choice(candidates) if candidates else prev_note

def make_bass_for_chord(chord, rhythm):
    # Repetitiver Bass: Grundton und Quinte, manchmal Oktave
    root = chord[0] - 12
    fifth = chord[2] - 12
    pattern = [root, fifth]
    bass = []
    for i, dur in enumerate(rhythm):
        if random.random() < 0.15:
            bass.append(root + 12)  # gelegentlich Oktave
        else:
            bass.append(pattern[i % 2])
    return bass

def make_voice_for_chord(chord, scale, prev_note, rhythm, is_melody=False, other_voice_rhythm=None):
    # Rhythmisches Wechselspiel: Wenn andere Stimme spielt, öfter Pause machen
    voice = []
    last_note = prev_note
    for i, dur in enumerate(rhythm):
        if other_voice_rhythm and i < len(other_voice_rhythm) and other_voice_rhythm[i] is not None:
            if random.random() < 0.5:
                voice.append(None)
                continue
        if random.random() < (0.15 if is_melody else 0.25):
            voice.append(None)
            continue
        note = pick_note(chord, scale, last_note)
        voice.append(note)
        last_note = note
    return voice, last_note

def write_track(track, notes, rhythm, channel=0, velocity=100):
    for note, dur in zip(notes, rhythm):
        if note is not None:
            track.append(Message('note_on', note=note, velocity=velocity, time=0, channel=channel))
            track.append(Message('note_off', note=note, velocity=velocity, time=dur, channel=channel))
        else:
            track.append(Message('note_off', note=60, velocity=0, time=dur, channel=channel))

def main():
    mid = MidiFile(ticks_per_beat=480)
    tempo = mido.bpm2tempo(120)
    mid.tracks.append(MidiTrack([mido.MetaMessage('set_tempo', tempo=tempo)]))

    melody = []
    background = []
    bass = []
    last_mel = 60
    last_bg = 64

    for bar, chord_name in enumerate(PROGRESSION):
        chord = CHORDS[chord_name]
        scale = C_MAJOR
        mel_rhythm = random.choice(RHYTHM_PATTERNS)
        bg_rhythm = random.choice(RHYTHM_PATTERNS)
        bass_rhythm = [240] * max(len(mel_rhythm), len(bg_rhythm))  # gleichmäßiger Bass

        # Rhythmisches Wechselspiel: Melodie und Begleitung achten aufeinander
        bar_melody, last_mel = make_voice_for_chord(chord, scale, last_mel, mel_rhythm, is_melody=True, other_voice_rhythm=bg_rhythm)
        bar_background, last_bg = make_voice_for_chord(chord, scale, last_bg, bg_rhythm, is_melody=False, other_voice_rhythm=mel_rhythm)
        bar_bass = make_bass_for_chord(chord, bass_rhythm)

        # Stimmen auf gleiche Länge bringen
        max_len = max(len(bar_melody), len(bar_background), len(bar_bass))
        bar_melody += [None] * (max_len - len(bar_melody))
        bar_background += [None] * (max_len - len(bar_background))
        bar_bass += [None] * (max_len - len(bar_bass))
        rhythm = [240] * max_len

        melody += bar_melody
        background += bar_background
        bass += bar_bass

    # Tracks schreiben
    melody_track = MidiTrack()
    melody_track.append(Message('program_change', program=81, time=0))
    write_track(melody_track, melody, [240]*len(melody), channel=0, velocity=110)
    mid.tracks.append(melody_track)

    background_track = MidiTrack()
    background_track.append(Message('program_change', program=87, time=0))
    write_track(background_track, background, [240]*len(background), channel=1, velocity=60)
    mid.tracks.append(background_track)

    bass_track = MidiTrack()
    bass_track.append(Message('program_change', program=38, time=0))
    write_track(bass_track, bass, [240]*len(bass), channel=2, velocity=70)
    mid.tracks.append(bass_track)

    mid.save(output_path)
    print(f"Fertig! Die Datei '{output_path}' wurde erzeugt.")

if __name__ == "__main__":
    main()
