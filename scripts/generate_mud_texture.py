from PIL import Image, ImageDraw, ImageFilter
import numpy as np
import random
import os

WIDTH, HEIGHT = 2048, 2048
TARGET_DIR = os.path.join("assets", "textures")
os.makedirs(TARGET_DIR, exist_ok=True)

base_color = (90, 60, 30)

# --- 1. Farbbild (mud_ground.png) ---
img = Image.new("RGB", (WIDTH, HEIGHT), base_color)
draw = ImageDraw.Draw(img)

def seamless_ellipse(draw, x, y, r, color):
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

# Schlamm-Flecken (mehr, größere, dominanter)
for _ in range(1800):  # Anzahl erhöht
    x = random.randint(0, WIDTH-1)
    y = random.randint(0, HEIGHT-1)
    r = random.randint(30, 140)  # größere Flecken
    if random.random() < 0.6:
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

# Moos-Flecken (weniger, dunkler, weniger leuchtend)
for _ in range(120):  # Weniger Moos-Flecken!
    x = random.randint(0, WIDTH-1)
    y = random.randint(0, HEIGHT-1)
    r = random.randint(30, 80)
    green = random.randint(30, 60)  # weniger leuchtend
    color = (
        base_color[0] - random.randint(10, 25),
        base_color[1] + green,
        base_color[2] - random.randint(10, 20),
    )
    # Moos abdunkeln (dunkleres, gedecktes Grün)
    color = (
        int(color[0] * 0.8),
        int(color[1] * 0.7 + 30),  # weniger leuchtend, mehr ins Olive
        int(color[2] * 0.7),
    )
    color = tuple(max(0, min(255, c)) for c in color)
    seamless_ellipse(draw, x, y, r, color)

# Weichzeichnen für matschigen Look
img = img.filter(ImageFilter.GaussianBlur(radius=4))

# Nahtloses Rauschen hinzufügen
pixels = img.load()
for i in range(WIDTH):
    for j in range(HEIGHT):
        noise = random.randint(-12, 12)
        r, g, b = pixels[i, j]
        pixels[i, j] = (
            max(0, min(255, r + noise)),
            max(0, min(255, g + noise)),
            max(0, min(255, b + noise)),
        )

img.save(os.path.join(TARGET_DIR, "mud_ground.png"))
print("mud_ground.png gespeichert!")

# --- 2. Normalmap (mud_ground_normal.png) ---
# Aus Farbbild eine Heightmap ableiten (Helligkeit = Höhe)
gray = img.convert("L")
height = np.asarray(gray).astype(np.float32) / 255.0

# Sobel-Operator für Normalmap
sobel_x = np.array([[-1,0,1],[-2,0,2],[-1,0,1]], dtype=np.float32)
sobel_y = np.array([[-1,-2,-1],[0,0,0],[1,2,1]], dtype=np.float32)
dx = np.zeros_like(height)
dy = np.zeros_like(height)
for i in range(1, HEIGHT-1):
    for j in range(1, WIDTH-1):
        dx[i,j] = np.sum(height[i-1:i+2, j-1:j+2] * sobel_x)
        dy[i,j] = np.sum(height[i-1:i+2, j-1:j+2] * sobel_y)

strength = 2.0  # Bumpmap-Stärke
normal = np.zeros((HEIGHT, WIDTH, 3), dtype=np.uint8)
for i in range(HEIGHT):
    for j in range(WIDTH):
        nx = -dx[i,j] * strength
        ny = -dy[i,j] * strength
        nz = 1.0
        length = np.sqrt(nx*nx + ny*ny + nz*nz)
        n = ((nx/length + 1) * 127.5, (ny/length + 1) * 127.5, (nz/length + 1) * 127.5)
        normal[i,j] = tuple(int(max(0, min(255, v))) for v in n)
normal_img = Image.fromarray(normal, "RGB")
normal_img.save(os.path.join(TARGET_DIR, "mud_ground_normal.png"))
print("mud_ground_normal.png gespeichert!")

# --- 3. Glossmap (mud_ground_gloss.png) ---
# Gloss = Helligkeit: Schlamm glänzt mehr (niedrige Roughness), Moos weniger (hohe Roughness)
# Wir nehmen die Grün-Komponente als "Moos-Detektor": viel Grün = matt, wenig Grün = glänzend

color = np.asarray(img).astype(np.float32)
# "Feuchter" Schlamm: wenig Grün, "trockenes" Moos: viel Grün
moos_mask = color[:,:,1] > (base_color[1] + 40)  # Schwellenwert für Moos

# Grundwert: noch glänzender (niedrigere Roughness)
gloss = np.full((HEIGHT, WIDTH), 24, dtype=np.uint8)  # 24 = noch feuchter/glänzender

# Moosbereiche matter machen (höhere Roughness)
gloss[moos_mask] = 200  # 200 = noch matter

gloss_img = Image.fromarray(gloss, "L")
gloss_img.save(os.path.join(TARGET_DIR, "mud_ground_gloss.png"))
print("mud_ground_gloss.png gespeichert!")
