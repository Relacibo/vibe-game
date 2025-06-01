from PIL import Image, ImageDraw, ImageFilter
import random
import os

WIDTH, HEIGHT = 2048, 2048

# Zielordner für Bevy
TARGET_PATH = os.path.join("assets", "textures", "mud_ground.png")
os.makedirs(os.path.dirname(TARGET_PATH), exist_ok=True)

# Grundfarbe für Schlamm
base_color = (90, 60, 30)

# Bild anlegen
img = Image.new("RGB", (WIDTH, HEIGHT), base_color)
draw = ImageDraw.Draw(img)

def seamless_ellipse(draw, x, y, r, color):
    # Zeichnet Ellipsen, die an den Rändern wiederholt werden (für Seamless)
    positions = [
        (x, y),
        (x + WIDTH, y),
        (x - WIDTH, y),
        (x, y + HEIGHT),
        (x, y - HEIGHT),
        (x + WIDTH, y + HEIGHT),
        (x - WIDTH, y - HEIGHT),
        (x + WIDTH, y - HEIGHT),
        (x - WIDTH, y + HEIGHT),
    ]
    for px, py in positions:
        draw.ellipse((px - r, py - r, px + r, py + r), fill=color)

# Schlamm-Flecken
for _ in range(1200):
    # Zufällige Position und Größe
    x = random.randint(0, WIDTH-1)
    y = random.randint(0, HEIGHT-1)
    r = random.randint(20, 120)
    # Zufällige Farbe (dunkler oder heller als Grundfarbe)
    if random.random() < 0.5:
        color = (
            base_color[0] - random.randint(10, 40),
            base_color[1] - random.randint(10, 30),
            base_color[2] - random.randint(5, 20),
        )
    else:
        color = (
            base_color[0] + random.randint(5, 20),
            base_color[1] + random.randint(5, 15),
            base_color[2] + random.randint(0, 10),
        )
    color = tuple(max(0, min(255, c)) for c in color)
    seamless_ellipse(draw, x, y, r, color)

# Moos-Flecken (grünliche Bereiche)
for _ in range(300):
    # Zufällige Position und Größe
    x = random.randint(0, WIDTH-1)
    y = random.randint(0, HEIGHT-1)
    r = random.randint(30, 100)
    green = random.randint(60, 120)
    color = (
        base_color[0] - random.randint(10, 30),
        base_color[1] + green,
        base_color[2] - random.randint(10, 20),
    )
    color = tuple(max(0, min(255, c)) for c in color)
    seamless_ellipse(draw, x, y, r, color)

# Weichzeichnen für matschigen Look
img = img.filter(ImageFilter.GaussianBlur(radius=4))

# Nahtloses Rauschen hinzufügen
pixels = img.load()
for i in range(WIDTH):
    for j in range(HEIGHT):
        # Rauschen, das an den Rändern wiederholt werden kann
        noise = (
            random.randint(-12, 12) +
            random.randint(-12, 12) * (i % 128 == 0 or j % 128 == 0)
        )
        r, g, b = pixels[i, j]
        pixels[i, j] = (
            max(0, min(255, r + noise)),
            max(0, min(255, g + noise)),
            max(0, min(255, b + noise)),
        )

img.save(TARGET_PATH)
print(f"{TARGET_PATH} gespeichert!")
