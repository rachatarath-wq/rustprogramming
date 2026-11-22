import wave, struct, math

RATE = 22050
CHORD_DUR = 2.0
DUR = 8.0  # 4 chords x 2s, loop
N = int(RATE * DUR)

chords = [
    [261.63, 329.63, 392.00],  # Cmaj
    [220.00, 261.63, 329.63],  # Amin
    [174.61, 220.00, 261.63],  # Fmaj
    [196.00, 246.94, 293.66],  # Gmaj
]
bass = [130.81, 110.00, 87.31, 98.00]   # C3 A2 F2 G2
melody = [659.25, 587.33, 523.25, 587.33, 659.25, 783.99, 880.00, 783.99]

samples = [0.0] * N

def add(freq, t0, dur, amp, attack=0.35, release=0.45):
    i0 = int(t0 * RATE)
    i1 = int((t0 + dur) * RATE)
    for i in range(i0, min(i1, N)):
        t = i / RATE - t0
        env = 1.0
        if t < attack:
            env = t / attack
        if t > dur - release:
            env = min(env, (dur - t) / release)
        if env <= 0:
            continue
        s = math.sin(2 * math.pi * freq * (i / RATE))
        samples[i] += amp * env * s

# pad (soft chords + shimmer octave)
for ci, chord in enumerate(chords):
    t0 = ci * CHORD_DUR
    for f in chord:
        add(f, t0, CHORD_DUR, 0.055)
        add(f * 2, t0, CHORD_DUR, 0.028)

# bass
for ci, f in enumerate(bass):
    add(f, ci * CHORD_DUR, CHORD_DUR, 0.10)

# gentle melody (1 note per second)
for mi, f in enumerate(melody):
    add(f, mi * 1.0, 0.95, 0.09, attack=0.12, release=0.35)

# normalize to 0.75 peak
peak = max(1e-6, max(abs(s) for s in samples))
scale = 0.75 / peak
frames = b''.join(
    struct.pack('<h', int(max(-1.0, min(1.0, s * scale)) * 32767))
    for s in samples
)

with wave.open('assets/bgm.wav', 'w') as w:
    w.setnchannels(1)
    w.setsampwidth(2)
    w.setframerate(RATE)
    w.writeframes(frames)

print('bgm.wav written:', len(frames), 'bytes,', DUR, 's, peak', peak)
