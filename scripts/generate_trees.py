import trimesh
import numpy as np
from PIL import Image, ImageDraw
import os

output_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'assets', 'models', 'trees'))
os.makedirs(output_dir, exist_ok=True)

def create_tree(trunk_height, trunk_radius, crown_radius, rotation_deg=0.0):
    # Stamm (Zylinder, Z-Achse nach oben)
    trunk = trimesh.creation.cylinder(radius=trunk_radius, height=trunk_height, sections=16)
    trunk.apply_translation([0, 0, trunk_height / 2])

    # Krone (Kugel, auf Stammspitze)
    crown = trimesh.creation.icosphere(subdivisions=2, radius=crown_radius)
    crown.apply_translation([0, 0, trunk_height + crown_radius])

    # UVs für Stamm (zylindrisch)
    theta = np.arctan2(trunk.vertices[:, 1], trunk.vertices[:, 0])
    trunk.visual.uv = np.column_stack([
        (theta / (2 * np.pi)) % 1.0,
        trunk.vertices[:, 2] / trunk_height
    ])

    # UVs für Krone (sphärisch)
    v = crown.vertices - crown.center_mass
    theta = np.arctan2(v[:, 1], v[:, 0])
    phi = np.arccos(v[:, 2] / crown_radius)
    crown.visual.uv = np.column_stack([
        (theta / (2 * np.pi)) % 1.0,
        phi / np.pi
    ])

    # Zufällige Rotation um Z (damit die Bäume unterschiedlich aussehen)
    if rotation_deg != 0.0:
        angle = np.deg2rad(rotation_deg)
        rot = trimesh.transformations.rotation_matrix(angle, [0, 0, 1])
        trunk.apply_transform(rot)
        crown.apply_transform(rot)

    return trunk, crown

def generate_trunk_texture(filename, size=256):
    img = Image.new("RGB", (size, size), (110, 70, 30))
    draw = ImageDraw.Draw(img)
    for i in range(20):
        x = np.random.randint(0, size)
        y1 = np.random.randint(0, size)
        y2 = np.random.randint(0, size)
        color = (90 + np.random.randint(0, 40), 60 + np.random.randint(0, 30), 20)
        draw.line((x, y1, x, y2), fill=color, width=2)
    img.save(filename)

def generate_crown_texture(filename, size=256):
    img = Image.new("RGB", (size, size), (40, 120, 40))
    draw = ImageDraw.Draw(img)
    for i in range(100):
        x = np.random.randint(0, size)
        y = np.random.randint(0, size)
        r = np.random.randint(8, 24)
        color = (30 + np.random.randint(0, 60), 100 + np.random.randint(0, 80), 30 + np.random.randint(0, 60))
        draw.ellipse((x, y, x+r, y+r), fill=color, outline=None)
    img.save(filename)

np.random.seed(42)
for i in range(12):
    # Varianz für jeden Baum
    trunk_height = np.random.uniform(1.2, 2.2)
    trunk_radius = np.random.uniform(0.12, 0.25)
    crown_radius = np.random.uniform(0.4, 0.7)
    rotation_deg = np.random.uniform(0, 360)
    trunk, crown = create_tree(trunk_height, trunk_radius, crown_radius, rotation_deg)

    # Speichern als OBJ
    trunk.export(os.path.join(output_dir, f"tree_{i}_trunk.obj"))
    crown.export(os.path.join(output_dir, f"tree_{i}_crown.obj"))

    # Texturen generieren
    generate_trunk_texture(os.path.join(output_dir, f"tree_{i}_trunk.png"))
    generate_crown_texture(os.path.join(output_dir, f"tree_{i}_crown.png"))

print("Alle Bäume und Texturen generiert!")