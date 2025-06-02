import numpy as np
import os

SCRIPT_NAME = "generate_roots"
TARGET_DIR = os.path.join("generated", SCRIPT_NAME)
os.makedirs(TARGET_DIR, exist_ok=True)

def create_root_obj(filename, segments=14, length=2.2, radius=0.09, style="normal", sides=8):
    verts = []
    faces = []
    for i in range(segments + 1):
        t = i / segments
        # Verschiedene Styles für interessante Formen
        if style == "spiral":
            angle = np.pi * 2 * t * 1.5 + np.random.uniform(-0.2, 0.2)
            x = np.cos(angle) * radius * (1 - t * 0.7)
            y = t * length + np.sin(t * np.pi) * 0.2
            z = np.sin(angle) * radius * (1 - t * 0.7)
        elif style == "bent":
            angle = np.pi * 2 * t + np.random.uniform(-0.2, 0.2)
            bend = np.sin(t * np.pi) * 0.5
            x = np.cos(angle) * radius * (1 - t * 0.7) + bend
            y = t * length
            z = np.sin(angle) * radius * (1 - t * 0.7)
        elif style == "split":
            angle = np.pi * 2 * t + np.random.uniform(-0.2, 0.2)
            split = 0.2 if t > 0.7 else 0.0
            x = np.cos(angle) * radius * (1 - t * 0.7) + split
            y = t * length
            z = np.sin(angle) * radius * (1 - t * 0.7) + split
        else:  # normal
            angle = np.pi * 2 * t + np.random.uniform(-0.2, 0.2)
            x = np.cos(angle) * radius * (1 - t * 0.7)
            y = t * length + np.random.uniform(-0.05, 0.05)
            z = np.sin(angle) * radius * (1 - t * 0.7)
        # Für jeden Abschnitt einen Kreis an Vertices erzeugen
        for s in range(sides):
            theta = 2 * np.pi * s / sides
            dx = np.cos(theta) * radius * 0.3
            dz = np.sin(theta) * radius * 0.3
            verts.append((x + dx, y, z + dz))
    # Faces erzeugen (Quads zwischen den Kreisen)
    for i in range(segments):
        for s in range(sides):
            curr = i * sides + s
            next = curr + sides
            next_s = i * sides + (s + 1) % sides
            next_next = next_s + sides
            faces.append((curr + 1, next + 1, next_next + 1, next_s + 1))
    with open(filename, "w") as f:
        for v in verts:
            f.write(f"v {v[0]:.4f} {v[1]:.4f} {v[2]:.4f}\n")
        for face in faces:
            f.write(f"f {' '.join(str(idx) for idx in face)}\n")

# Hauptwurzeln
styles = ["normal", "spiral", "bent", "split"]
for i in range(5):
    style = np.random.choice(styles)
    create_root_obj(
        os.path.join(TARGET_DIR, f"root_{i}.obj"),
        segments=np.random.randint(10, 18),
        length=np.random.uniform(1.5, 2.5),
        radius=np.random.uniform(0.07, 0.12),
        style=style
    )

# Splitter (kleine, kurze Stücke)
for i in range(4):
    create_root_obj(
        os.path.join(TARGET_DIR, f"root_splitter_{i}.obj"),
        segments=np.random.randint(4, 7),
        length=np.random.uniform(0.3, 0.7),
        radius=np.random.uniform(0.03, 0.06),
        style="normal"
    )

print(f"Mehrere Wurzel-Meshes generiert in {TARGET_DIR}")
