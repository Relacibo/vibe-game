import numpy as np
import soundfile as sf

samplerate = 44100
duration = 5.0  # Gesamtlänge jetzt 5 Sekunden
t = np.linspace(0, duration, int(samplerate * duration), False)

# --- Entwurzelsound wie gehabt ---
crack_buildup = np.zeros_like(t)
buildup_times = np.linspace(0.05, 0.5, 12) + np.random.uniform(-0.01, 0.01, 12)
for i, ct in enumerate(buildup_times):
    idx = int(ct * samplerate)
    width = np.random.randint(80, 220)
    amp = 0.5 + 0.5 * (i / len(buildup_times))
    if idx + width < len(crack_buildup):
        crack_buildup[idx:idx+width] += np.hanning(width) * amp

bass_start = 0.5
bass_t = t - bass_start
bass_t[bass_t < 0] = 0
freq = 28
bass = 2.2 * np.sin(2 * np.pi * freq * bass_t) * np.exp(-bass_t / 0.85)

creak = (
    0.8 * np.sin(2 * np.pi * 90 * t + np.sin(2 * np.pi * 2.2 * t)) * np.exp(-bass_t / 0.33)
    + 0.6 * np.sin(2 * np.pi * 61 * t + np.sin(2 * np.pi * 1.1 * t)) * np.exp(-bass_t / 0.45)
    + 0.4 * np.sin(2 * np.pi * 140 * t + np.sin(2 * np.pi * 3.7 * t)) * np.exp(-bass_t / 0.19)
)
cracks = np.zeros_like(t)
num_cracks = 28
for ct in np.random.uniform(0.55, 1.3, num_cracks):
    idx = int(ct * samplerate)
    width = np.random.randint(60, 200)
    amp = np.random.uniform(0.7, 1.3)
    if idx + width < len(cracks):
        cracks[idx:idx+width] += np.hanning(width) * amp

aftershock = 0.5 * np.sin(2 * np.pi * 38 * t + np.sin(2 * np.pi * 0.7 * t)) * np.exp(-((t-1.2)/0.38)**2)
noise = np.random.randn(len(t)) * np.exp(-bass_t / 0.19) * 0.06

entwurzel_sound = (
    0.7 * crack_buildup +
    1.0 * bass +
    0.7 * creak +
    0.7 * cracks +
    0.5 * aftershock +
    noise
)

# --- Dreckregen mit langem Fade-out ---
dirt_start = 0.5
fade_in_time = 2.5
hold_time = 0.5
fade_out_time = 1.5  # Angepasst, damit dirt_end = 5.0s
dirt_len = fade_in_time + hold_time + fade_out_time
dirt_N = int(dirt_len * samplerate)
dirt_t = np.linspace(0, dirt_len, dirt_N, endpoint=False)

fade = np.ones_like(dirt_t)
fade_in_N = int(fade_in_time * samplerate)
fade_out_N = int(fade_out_time * samplerate)

# Exponentieller Fade-in (fürs Ohr linear)
fade[:fade_in_N] = np.logspace(-2, 0, fade_in_N, base=10)
# Hold
fade[fade_in_N: dirt_N - fade_out_N] = 1.0
# Exponentieller Fade-out (fürs Ohr linear)
if fade_out_N > 0:
    fade[-fade_out_N:] = np.logspace(0, -2, fade_out_N, base=10)

dirt_rain = np.random.randn(dirt_N) * np.exp(-dirt_t / 1.5) * 0.7 * fade
for ct in np.random.uniform(0.0, dirt_len, 22):
    idx = int(ct * samplerate)
    width = np.random.randint(80, 200)
    amp = np.random.uniform(0.3, 0.7)
    if idx + width < len(dirt_rain):
        dirt_rain[idx:idx+width] += np.hanning(width) * amp

dirt_full = np.zeros_like(t)
dirt_start_idx = int(dirt_start * samplerate)
dirt_end_idx = dirt_start_idx + dirt_N
if dirt_end_idx > len(dirt_full):
    dirt_end_idx = len(dirt_full)
    dirt_rain = dirt_rain[:dirt_end_idx - dirt_start_idx]
dirt_full[dirt_start_idx:dirt_end_idx] = dirt_rain

# --- Mischung ---
sound = entwurzel_sound + 0.8 * dirt_full

# Normalisieren
sound *= 0.98 / np.max(np.abs(sound))

sf.write("assets/sounds/stake.wav", sound, samplerate)
print("Stake sound gespeichert unter assets/sounds/stake.wav")
