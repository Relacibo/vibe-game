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
    "ii": [62, 65, 69],      # D F A
    "iii":[64, 67, 71],      # E G B
    "IV": [65, 69, 72],      # F A C
    "V":  [67, 71, 74],      # G B D
    "vi": [69, 72, 76],      # A C E
    "V7": [67, 71, 74, 77],  # G B D F
}

PROGRESSIONS = [
    ["I", "IV", "V", "I"],
    ["I", "vi", "IV", "V"],
    ["I", "V", "vi", "IV"],
    ["I", "IV", "I", "V"],
    ["I", "V7", "I", "IV", "V", "I"]
]

def choose_progression():
    return random.choice(PROGRESSIONS)

def make_melody_for_chord(chord, prev_note=None, length=4):
    # Melodie bleibt auf Akkordtönen, mit gelegentlichen Durchgangsnoten
    melody = []
    chord_tones = chord
    scale = C_MAJOR
    for i in range(length):
        if i == 0 and prev_note is not None:
            # Schrittweise Bewegung von vorherigem Ton
            options = [n for n in chord_tones if abs(n - prev_note) <= 5]
            note = random.choice(options) if options else random.choice(chord_tones)
        elif random.random() < 0.8:
            note = random.choice(chord_tones)
        else:
            # Durchgangsnote zwischen zwei Akkordtönen
            idx = random.randint(0, len(chord_tones)-2)
            low = chord_tones[idx]
            high = chord_tones[idx+1]
            between = [n for n in scale if low < n < high]
            note = random.choice(between) if between else random.choice(chord_tones)
        melody.append(note)
        prev_note = note
    return melody

def make_colloratur(bar_root, length=8):
    # Durchgehende Achtelnoten, auf Skala, um bar_root herum
    idx = C_MAJOR.index(bar_root)
    notes = []
    for i in range(length):
        offset = random.choice([-2, -1, 0, 1, 2])
        nidx = max(0, min(len(C_MAJOR)-1, idx + offset))
        notes.append(C_MAJOR[nidx] + 12)
    return notes

def make_bass_for_chord(chord, length=4):
    root = chord[0]
    fifth = chord[2]
    bass = []
    for i in range(length):
        if i == 0:
            bass.append(root)
        elif i == length-1 and random.random() < 0.3:
            bass.append(root + 12)  # Oktave für Abschluss
        else:
            bass.append(fifth if random.random() < 0.5 else root)
    return bass

def make_chord_track(chord, length=4):
    notes = []
    for i in range(length):
        if i % 2 == 0:
            notes.append(chord)
        else:
            notes.append([])
    return notes

def write_melody(track, melody, rhythm, channel=0, velocity=100):
    for note, dur in zip(melody, rhythm):
        if note is not None:
            track.append(Message('note_on', note=note, velocity=velocity, time=0, channel=channel))
            track.append(Message('note_off', note=note, velocity=velocity, time=dur, channel=channel))
        else:
            track.append(Message('note_off', note=60, velocity=0, time=dur, channel=channel))

def write_bass(track, bass, rhythm, channel=1, velocity=80):
    for note, dur in zip(bass, rhythm):
        if note is not None:
            track.append(Message('note_on', note=note-12, velocity=velocity, time=0, channel=channel))
            track.append(Message('note_off', note=note-12, velocity=velocity, time=dur, channel=channel))
        else:
            track.append(Message('note_off', note=48, velocity=0, time=dur, channel=channel))

def write_chords(track, chords, rhythm, channel=2, velocity=60):
    for chord, dur in zip(chords, rhythm):
        if chord:
            for note in chord:
                track.append(Message('note_on', note=note, velocity=velocity, time=0, channel=channel))
            for note in chord:
                track.append(Message('note_off', note=note, velocity=velocity, time=dur, channel=channel))
        else:
            track.append(Message('note_off', note=60, velocity=0, time=dur, channel=channel))

def main():
    mid = MidiFile(ticks_per_beat=480)
    tempo = mido.bpm2tempo(132)
    mid.tracks.append(MidiTrack([mido.MetaMessage('set_tempo', tempo=tempo)]))

    progression = choose_progression() * 8  # ca. 32 Takte
    melody = []
    bass = []
    chords = []
    rhythm = []
    prev_note = None

    for i, chord_name in enumerate(progression):
        chord = CHORDS[chord_name]
        # Rhythmus: Abwechslung
        if random.random() < 0.15:
            # Colloratur-Passage (durchgehende Achtel)
            bar_rhythm = [240]*8
            bar_melody = make_colloratur(chord[0], length=8)
            bar_bass = make_bass_for_chord(chord, length=8)
            bar_chords = make_chord_track(chord, length=8)
        elif random.random() < 0.1:
            # Synkope
            bar_rhythm = [360, 120, 480, 480]
            bar_melody = make_melody_for_chord(chord, prev_note, length=4)
            bar_bass = make_bass_for_chord(chord, length=4)
            bar_chords = make_chord_track(chord, length=4)
        else:
            # Standard
            bar_rhythm = [240, 240, 480, 480]
            bar_melody = make_melody_for_chord(chord, prev_note, length=4)
            bar_bass = make_bass_for_chord(chord, length=4)
            bar_chords = make_chord_track(chord, length=4)
        prev_note = bar_melody[-1]
        rhythm += bar_rhythm
        melody += bar_melody
        bass += bar_bass
        chords += bar_chords

    # Tracks schreiben
    melody_track = MidiTrack()
    write_melody(melody_track, melody, rhythm)
    mid.tracks.append(melody_track)

    bass_track = MidiTrack()
    write_bass(bass_track, bass, rhythm)
    mid.tracks.append(bass_track)

    chords_track = MidiTrack()
    write_chords(chords_track, chords, rhythm)
    mid.tracks.append(chords_track)

    mid.save(output_path)
    print(f"Fertig! Die Datei '{output_path}' wurde erzeugt.")

if __name__ == "__main__":
    main()
