import mido
from mido import Message, MidiFile, MidiTrack
import random
import os

script_dir = os.path.dirname(os.path.abspath(__file__))
output_dir = os.path.join(script_dir, "..", "generated", "generate_8bit_music")
os.makedirs(output_dir, exist_ok=True)
output_path = os.path.join(output_dir, "vibe_8bit_theme.mid")

C_MAJOR = [60, 62, 64, 65, 67, 69, 71, 72]  # C D E F G A B C
CHORDS = {
    "I":  [60, 64, 67],      # C E G
    "IV": [65, 69, 72],      # F A C
    "V":  [67, 71, 74],      # G B D
    "vi": [69, 72, 76],      # A C E
    "V7": [67, 71, 74, 77],  # G B D F
}

PROGRESSION = ["I", "IV", "V", "I", "vi", "IV", "V", "I"] * 4

RHYTHM_PATTERNS = [
    [480, 480, 480, 480],
    [240, 240, 480, 480],
    [360, 120, 360, 120, 480],
    [320, 160, 320, 160, 320],
    [480, 240, 240, 480],
    [720, 240, 480],
    [240, 240, 240, 240, 480],
]

def make_melody_motif(scale, start_note, length=4):
    motif = [start_note]
    for _ in range(length - 1):
        last = motif[-1]
        options = [n for n in scale if abs(n - last) in [2, 4] and n != last]
        if not options:
            options = scale
        motif.append(random.choice(options))
    return motif

def make_melody_for_chord(chord, scale, prev_note, length, rhythm):
    melody = []
    last_note = prev_note
    for i in range(length):
        if i == 0 and last_note is not None:
            options = [n for n in chord if abs(n - last_note) <= 4]
            note = random.choice(options) if options else random.choice(chord)
        elif random.random() < 0.8:
            step_options = [n for n in chord if abs(n - last_note) in [2, 4]]
            note = random.choice(step_options) if step_options else random.choice(chord)
        else:
            idx = scale.index(last_note) if last_note in scale else 0
            if idx > 0 and idx < len(scale) - 1:
                note = scale[idx + random.choice([-1, 1])]
            else:
                note = last_note
        melody.append(note)
        last_note = note
    return melody, last_note

def make_background_for_chord(chord, length, rhythm):
    # Dezente Akkordstimme: meist Haltenoten oder punktierte Noten, selten Synkopen
    bg = []
    for i in range(length):
        if i == 0:
            # Akkordton (Terz oder Quinte, nicht Grundton)
            note = random.choice(chord[1:])
            bg.append(note)
        elif random.random() < 0.2:
            # Pause für Luft
            bg.append(None)
        else:
            # Haltenote oder Wiederholung
            bg.append(bg[-1] if bg else random.choice(chord[1:]))
    return bg

def make_bass_for_chord(chord, length, rhythm):
    # Bass: Grundton, Quinte, Grundton, Oktave
    root = chord[0] - 12
    fifth = chord[2] - 12
    notes = []
    for i in range(length):
        if i % 4 == 0:
            notes.append(root)
        elif i % 4 == 2:
            notes.append(fifth)
        elif random.random() < 0.2:
            notes.append(root + 12)
        else:
            notes.append(root)
    return notes

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
    rhythm = []
    last_note = 60  # C4

    # Melodie-orientierter Anfang (nur Melodie, 4 Takte, motivisch)
    motif = make_melody_motif(C_MAJOR, last_note, length=4)
    for _ in range(2):
        pattern = random.choice(RHYTHM_PATTERNS)
        melody += motif[:len(pattern)]
        background += [None] * len(pattern)
        bass += [None] * len(pattern)
        rhythm += pattern
        last_note = melody[-1]

    motif2 = [n + random.choice([-2, 0, 2]) for n in motif]
    for _ in range(2):
        pattern = random.choice(RHYTHM_PATTERNS)
        melody += motif2[:len(pattern)]
        background += [None] * len(pattern)
        bass += [None] * len(pattern)
        rhythm += pattern
        last_note = melody[-1]

    # Jetzt Akkorde und Bass dazu, Melodie bleibt im Vordergrund
    for chord_name in PROGRESSION:
        chord = CHORDS[chord_name]
        scale = C_MAJOR
        pattern = random.choice(RHYTHM_PATTERNS)
        bar_len = len(pattern)
        bar_melody, last_note = make_melody_for_chord(chord, scale, last_note, bar_len, pattern)
        bar_background = make_background_for_chord(chord, bar_len, pattern)
        bar_bass = make_bass_for_chord(chord, bar_len, pattern)
        melody += bar_melody
        background += bar_background
        bass += bar_bass
        rhythm += pattern

    # Tracks schreiben
    melody_track = MidiTrack()
    melody_track.append(Message('program_change', program=81, time=0))      # Lead 1 (square)
    write_track(melody_track, melody, rhythm, channel=0, velocity=110)
    mid.tracks.append(melody_track)

    background_track = MidiTrack()
    background_track.append(Message('program_change', program=82, time=0))  # Lead 2 (sawtooth)
    write_track(background_track, background, rhythm, channel=1, velocity=60)
    mid.tracks.append(background_track)

    bass_track = MidiTrack()
    bass_track.append(Message('program_change', program=38, time=0))        # Synth Bass 1
    write_track(bass_track, bass, rhythm, channel=2, velocity=70)
    mid.tracks.append(bass_track)

    mid.save(output_path)
    print(f"Fertig! Die Datei '{output_path}' wurde erzeugt.")

if __name__ == "__main__":
    main()
