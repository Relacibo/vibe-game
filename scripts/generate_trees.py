import trimesh
import numpy as np
from PIL import Image, ImageDraw
import os
import json

output_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'assets', 'models', 'trees'))
os.makedirs(output_dir, exist_ok=True)

def create_branch(base, length, radius, angle, tilt):
    # Erzeuge einen Ast als Zylinder
    branch = trimesh.creation.cylinder(radius=radius, height=length, sections=12)
    branch.apply_translation([0, 0, length / 2])
    # Rotation für Richtung
    rot1 = trimesh.transformations.rotation_matrix(angle, [0, 0, 1])  # um Z
    rot2 = trimesh.transformations.rotation_matrix(tilt, [1, 0, 0])   # um X
    branch.apply_transform(rot2)
    branch.apply_transform(rot1)
    # Ast an Basis verschieben
    branch.apply_translation(base)
    return branch

def create_tree(trunk_height, trunk_radius, crown_radius, rotation_deg=0.0, branch_count=0, extra_crowns=0):
    # Stamm (Zylinder, Z-Achse nach oben)
    trunk = trimesh.creation.cylinder(radius=trunk_radius, height=trunk_height, sections=16)
    trunk.apply_translation([0, 0, trunk_height / 2])

    # Krone (Kugel, auf Stammspitze)
    crown = trimesh.creation.icosphere(subdivisions=2, radius=crown_radius)
    crown.apply_translation([0, 0, trunk_height + crown_radius])

    # Äste erzeugen
    branches = []
    crowns = [crown]
    for b in range(branch_count):
        base_height = np.random.uniform(trunk_height * 0.3, trunk_height * 0.9)
        base = [0, 0, base_height]
        length = np.random.uniform(0.3, 0.7) * trunk_height
        radius = np.random.uniform(0.04, 0.09) * trunk_radius
        angle = np.random.uniform(0, 2 * np.pi)
        tilt = np.random.uniform(np.pi / 6, np.pi / 3)
        branch = create_branch(base, length, radius, angle, tilt)
        branches.append(branch)

    # Zusätzliche Kronen mit Ästen verbinden
    for c in range(extra_crowns):
        base_height = np.random.uniform(trunk_height * 0.5, trunk_height * 0.95)
        base = [0, 0, base_height]
        length = np.random.uniform(0.5, 1.0) * trunk_height * 0.5
        radius = np.random.uniform(0.05, 0.12) * trunk_radius
        angle = np.random.uniform(0, 2 * np.pi)
        tilt = np.random.uniform(np.pi / 6, np.pi / 2)
        branch = create_branch(base, length, radius, angle, tilt)
        end = branch.vertices[np.argmax(branch.vertices[:, 2])]
        crown2 = trimesh.creation.icosphere(subdivisions=2, radius=np.random.uniform(0.5, 0.9) * crown_radius)
        crown2.apply_translation(end)
        branches.append(branch)
        crowns.append(crown2)

    # UVs für Stamm (zylindrisch)
    z_min = trunk.vertices[:, 2].min()
    z_max = trunk.vertices[:, 2].max()
    theta = np.arctan2(trunk.vertices[:, 1], trunk.vertices[:, 0])
    trunk.visual.uv = np.column_stack([
        (theta / (2 * np.pi)) % 1.0,
        (trunk.vertices[:, 2] - z_min) / (z_max - z_min)
    ])

    # UVs für Kronen (sphärisch)
    for c in crowns:
        v = c.vertices - c.center_mass
        theta = np.arctan2(v[:, 1], v[:, 0])
        phi = np.arccos(v[:, 2] / c.bounding_sphere.primitive.radius)
        c.visual.uv = np.column_stack([
            (theta / (2 * np.pi)) % 1.0,
            phi / np.pi
        ])

    # UVs für Äste (zylindrisch)
    for b in branches:
        theta = np.arctan2(b.vertices[:, 1], b.vertices[:, 0])
        b.visual.uv = np.column_stack([
            (theta / (2 * np.pi)) % 1.0,
            b.vertices[:, 2] / (b.bounds[1][2] - b.bounds[0][2] + 1e-6)
        ])

    # Zufällige Rotation um Z (damit die Bäume unterschiedlich aussehen)
    if rotation_deg != 0.0:
        angle = np.deg2rad(rotation_deg)
        rot = trimesh.transformations.rotation_matrix(angle, [0, 0, 1])
        trunk.apply_transform(rot)
        for c in crowns:
            c.apply_transform(rot)
        for b in branches:
            b.apply_transform(rot)

    return trunk, crowns, branches

def generate_trunk_texture(filename, size=256):
    img = Image.new("RGB", (size, size), (np.random.randint(90, 130), np.random.randint(60, 90), np.random.randint(30, 50)))
    draw = ImageDraw.Draw(img)
    # Grundmaserung (vertikale Linien)
    for i in range(size//8, size, np.random.randint(6, 16)):
        color = (
            np.random.randint(80, 140),
            np.random.randint(50, 90),
            np.random.randint(20, 40)
        )
        waviness = np.random.uniform(2, 8)
        for y in range(size):
            x = int(i + np.sin(y / waviness) * np.random.uniform(1, 4))
            draw.point((x, y), fill=color)
    # Astnarben (ovale Flecken)
    for _ in range(np.random.randint(3, 8)):
        x = np.random.randint(size//8, size*7//8)
        y = np.random.randint(size//8, size*7//8)
        w = np.random.randint(10, 30)
        h = np.random.randint(6, 18)
        color = (70, 40, 20)
        draw.ellipse((x, y, x+w, y+h), outline=color, width=2)
    # Moos und Flechten
    for _ in range(np.random.randint(8, 18)):
        x = np.random.randint(0, size)
        y = np.random.randint(size//2, size)
        r = np.random.randint(4, 14)
        color = (np.random.randint(60, 100), np.random.randint(120, 180), np.random.randint(30, 60))
        draw.ellipse((x, y, x+r, y+r), fill=color, outline=None)
    # Risse (dunkle Linien)
    for _ in range(np.random.randint(8, 16)):
        x = np.random.randint(0, size)
        y1 = np.random.randint(0, size)
        y2 = y1 + np.random.randint(-10, 10)
        color = (40, 30, 20)
        draw.line((x, y1, x, y2), fill=color, width=1)
    img.save(filename)

def generate_branch_texture(filename, size=256):
    img = Image.new("RGB", (size, size), (110, 80, 50))
    draw = ImageDraw.Draw(img)
    # Maserung
    for i in range(size//10, size, np.random.randint(8, 18)):
        color = (
            np.random.randint(90, 130),
            np.random.randint(60, 90),
            np.random.randint(30, 50)
        )
        waviness = np.random.uniform(2, 8)
        for y in range(size):
            x = int(i + np.sin(y / waviness) * np.random.uniform(1, 3))
            draw.point((x, y), fill=color)
    # Flechten
    for _ in range(np.random.randint(4, 10)):
        x = np.random.randint(0, size)
        y = np.random.randint(size//2, size)
        r = np.random.randint(3, 10)
        color = (np.random.randint(80, 120), np.random.randint(160, 200), np.random.randint(40, 80))
        draw.ellipse((x, y, x+r, y+r), fill=color, outline=None)
    img.save(filename)

def generate_crown_texture(filename, size=256):
    from PIL import Image, ImageDraw
    import numpy as np

    # Hintergrund: sattes, mittleres Grün
    base_green = np.random.randint(80, 140)
    img = Image.new("RGB", (size, size), (base_green, np.random.randint(120, 180), base_green))
    draw = ImageDraw.Draw(img)

    # Viele Blätter (verschiedene Grüntöne, Größen, Formen)
    for _ in range(260):
        x = np.random.randint(0, size)
        y = np.random.randint(0, size)
        r = np.random.randint(8, 28)
        g = np.random.randint(80, 200)
        color = (
            min(255, g + np.random.randint(-30, 60)),
            min(255, 120 + np.random.randint(-20, 100)),
            min(255, g + np.random.randint(-30, 60))
        )
        # Manchmal ovale, manchmal runde Blätter
        if np.random.rand() < 0.7:
            draw.ellipse((x, y, x+r, y+r), fill=color, outline=None)
        else:
            draw.ellipse((x, y, x+r, y+r//2), fill=color, outline=None)

    # Kleine Äste (dunkle Linien)
    for _ in range(22):
        x1 = np.random.randint(0, size)
        y1 = np.random.randint(size//3, size)
        x2 = x1 + np.random.randint(-30, 30)
        y2 = y1 + np.random.randint(-40, 10)
        color = (60, 40, 20)
        draw.line((x1, y1, x2, y2), fill=color, width=np.random.randint(2, 5))

    # Schatten (dunkle Tupfer)
    for _ in range(70):
        x = np.random.randint(0, size)
        y = np.random.randint(size//4, size)
        r = np.random.randint(10, 32)
        color = (
            np.random.randint(20, 60),
            np.random.randint(40, 80),
            np.random.randint(20, 60)
        )
        draw.ellipse((x, y, x+r, y+r), fill=color, outline=None)

    img.save(filename)

def generate_trunk_bump(filename, size=256):
    img = Image.new("L", (size, size), 128)
    draw = ImageDraw.Draw(img)
    # Vertikale Maserung
    for i in range(size//8, size, np.random.randint(6, 16)):
        waviness = np.random.uniform(2, 8)
        for y in range(size):
            x = int(i + np.sin(y / waviness) * np.random.uniform(1, 4))
            val = np.random.randint(100, 180)
            draw.point((x, y), fill=val)
    # Astnarben als Vertiefungen
    for _ in range(np.random.randint(3, 8)):
        x = np.random.randint(size//8, size*7//8)
        y = np.random.randint(size//8, size*7//8)
        w = np.random.randint(10, 30)
        h = np.random.randint(6, 18)
        draw.ellipse((x, y, x+w, y+h), fill=90)
    img.save(filename)

def generate_crown_bump(filename, size=256):
    from PIL import Image, ImageDraw
    import numpy as np

    img = Image.new("L", (size, size), 128)
    draw = ImageDraw.Draw(img)

    # "Löcher" (Vertiefungen, z.B. Lücken im Blätterdach)
    for _ in range(30):
        x = np.random.randint(0, size)
        y = np.random.randint(0, size)
        r = np.random.randint(10, 32)
        val = np.random.randint(40, 90)  # dunkler = tiefer
        draw.ellipse((x, y, x+r, y+r), fill=val)

    # Erhebungen (z.B. dichte Blattbüschel)
    for _ in range(40):
        x = np.random.randint(0, size)
        y = np.random.randint(0, size)
        r = np.random.randint(6, 18)
        val = np.random.randint(160, 210)  # heller = höher
        draw.ellipse((x, y, x+r, y+r), fill=val)

    # Unregelmäßige Strukturen (kleine Tupfer)
    for _ in range(80):
        x = np.random.randint(0, size)
        y = np.random.randint(0, size)
        r = np.random.randint(2, 8)
        val = np.random.randint(100, 160)
        draw.ellipse((x, y, x+r, y+r), fill=val)

    img.save(filename)

def ensure_uv_visual(mesh):
    # Falls das Mesh keine TextureVisuals hat, erstelle sie
    if not isinstance(mesh.visual, trimesh.visual.texture.TextureVisuals):
        mesh.visual = trimesh.visual.texture.TextureVisuals(uv=mesh.visual.uv)

np.random.seed(42)
for i in range(12):
    # Varianz für jeden Baum
    trunk_height = np.random.uniform(1.2, 2.2)
    trunk_radius = np.random.uniform(0.12, 0.25)
    crown_radius = np.random.uniform(0.4, 0.7)
    rotation_deg = np.random.uniform(0, 360)
    branch_count = np.random.randint(2, 6)
    extra_crowns = np.random.randint(0, 3)
    trunk, crowns, branches = create_tree(trunk_height, trunk_radius, crown_radius, rotation_deg, branch_count, extra_crowns)

    ensure_uv_visual(trunk)
    trunk.export(os.path.join(output_dir, f"tree_{i}_trunk.obj"))
    for j, c in enumerate(crowns):
        ensure_uv_visual(c)
        c.export(os.path.join(output_dir, f"tree_{i}_crown{j}.obj"))
    for j, b in enumerate(branches):
        ensure_uv_visual(b)
        b.export(os.path.join(output_dir, f"tree_{i}_branch{j}.obj"))

    # Texturen generieren
    generate_trunk_texture(os.path.join(output_dir, f"tree_{i}_trunk.png"))
    generate_branch_texture(os.path.join(output_dir, f"tree_{i}_branch.png"))
    generate_crown_texture(os.path.join(output_dir, f"tree_{i}_crown.png"))
    generate_trunk_bump(os.path.join(output_dir, f"tree_{i}_trunk_bump.png"))
    generate_crown_bump(os.path.join(output_dir, f"tree_{i}_crown_bump.png"))

    # Collider-Informationen speichern
    crown_center_y = trunk_height + crown_radius
    crown_height = np.random.uniform(0.3, 0.7) * trunk_height
    tree_info = {
        "trunk": {
            "center": [0, 0.6, 0],
            "radius": trunk_radius,
            "height": trunk_height,
        },
        "crown": {
            "center": [0, crown_center_y, 0],
            "radius": crown_radius,
            "height": crown_height,
        }
    }
    with open(os.path.join(output_dir, f"tree_{i}_collider.json"), "w") as f:
        json.dump(tree_info, f)

print("Alle Bäume und Texturen generiert!")
