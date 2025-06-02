# 🛠️ Scripts für Vibe Game

Hier findest du alle Tools zur automatischen Generierung und Konvertierung der Baum-Assets, Texturen, Bumpmaps, Collider-Daten und Musik für das chaotischste Baumspiel der Welt! 🌳💥

---

## 🎵 8-Bit Musik generieren

**Script:** `music/generate_8bit_music.py`

**Voraussetzungen:**  
- Python 3  
- Pakete: `mido`, `numpy`, `python-rtmidi`  
  Installation:  
  ```sh
  pip install mido numpy python-rtmidi
  ```

**Verwendung:**  
```sh
python3 music/generate_8bit_music.py
```
- Erstellt ein zufälliges, mehrstimmiges 8-Bit-MIDI-Stück (`generated/generate_8bit_music/vibe_8bit_theme.mid`), das du in einen OGG/WAV umwandeln und direkt im Spiel verwenden kannst.

**Umwandlung in OGG/WAV mit MuseScore:**  
1. Öffne die Datei `generated/generate_8bit_music/vibe_8bit_theme.mid` in [MuseScore](https://musescore.org/de).
2. Wähle im Menü **Datei → Exportieren**.
3. Wähle als Format z.B. **OGG Vorbis** oder **WAV**.
4. Klicke auf **Exportieren** und speichere die Datei.
5. Die exportierte Audiodatei kannst du direkt im Spiel verwenden!

_Tipp: In MuseScore kannst du auch die Instrumente auf typische 8-Bit-Sounds (z.B. Square, Pulse, Synth) umstellen, um den Vibe zu verstärken.  
Dazu: Rechtsklick auf die Spur → **Eigenschaften Notenzeile/Instrument...** → Im sich öffnenden Fenster kannst du unter „Instrument“ ein anderes auswählen, z.B. „Synth Lead“ oder „Square Lead“.  
Für noch mehr Retro-Feeling kannst du auch eigene Soundfonts mit Chiptune-Instrumenten laden!_

---

## 🌲 Bäume & Texturen generieren

**Script:** `generate_trees.py`

**Voraussetzungen:**  
- Python 3  
- Pakete: `trimesh`, `numpy`, `Pillow`, `scipy`  
  Installation:  
  ```sh
  pip install trimesh numpy pillow scipy
  ```

**Verwendung:**  
```sh
python3 generate_trees.py
```
- Legt alle generierten Dateien im Ordner `generated/generate_trees/` ab:
  - Modelle (`tree_X_trunk.obj`, `tree_X_crown0.obj`, ...)
  - Texturen (`tree_X_trunk.png`, `tree_X_crown.png`, ...)
  - Bumpmaps (`tree_X_trunk_bump.png`, `tree_X_crown_bump.png`)
  - Collider-Infos als JSON (`tree_X_collider.json`)

---

## 🔄 OBJ-Bäume zu GLB konvertieren

**Script:** `batch_obj_to_gltf.py`

**Voraussetzungen:**  
- [Blender](https://www.blender.org/download/) (empfohlen: offizielle Version, nicht Flatpak/Snap)

**Verwendung:**  
```sh
blender --background --python batch_obj_to_gltf.py
```
- Liest alle Modelle und Texturen aus `generated/generate_trees/` und exportiert die GLB-Dateien nach `assets/models/trees/`.

---

## 🟫 Schlammige Bodentexturen generieren

**Script:** `generate_mud_texture.py`

**Voraussetzungen:**  
- Python 3  
- Pakete: `Pillow`, `numpy`  
  Installation:  
  ```sh
  pip install pillow numpy
  ```

**Verwendung:**  
```sh
python3 generate_mud_texture.py
```
- Erstellt drei nahtlose Texturen direkt im Ordner `assets/textures/`:
  - **mud_ground.png**: Farbbild (Albedo) des Bodens
  - **mud_ground_normal.png**: Normalmap/Bumpmap für plastische Beleuchtung
  - **mud_ground_gloss.png**: Glossmap (Glanz/Feuchtigkeit, Moos ist matter)

**Verwendung in Bevy:**  
- `mud_ground.png` als `base_color_texture`
- `mud_ground_normal.png` als `normal_map_texture`
- `mud_ground_gloss.png` z.B. als `metallic_roughness_texture` oder für eigene Shader

_Tipp: Passe die Parameter im Skript an, um mehr oder weniger Moos, größere Flecken oder andere Farben zu erhalten!_

---

## 🌱 Wurzel-Meshes & Texturen generieren

**Script:** `generate_tree_root_particles.py`

**Voraussetzungen:**  
- Python 3  
- Pakete: `numpy`, `Pillow`  
  Installation:  
  ```sh
  pip install numpy pillow
  ```

**Verwendung:**  
```sh
python3 generate_tree_root_particles.py
```
- Erzeugt mehrere prozedurale Wurzel-Meshes (`root_*.obj`, `root_splitter_*.obj`) und eine passende Textur (`root_diffuse.png`) im Ordner `generated/generate_roots/`.

---

## 🪄 Wurzel-OBJ zu GLB konvertieren

**Script:** `root_obj_to_gltf.py`

**Voraussetzungen:**  
- [Blender](https://www.blender.org/download/) (empfohlen: offizielle Version, nicht Flatpak/Snap)

**Verwendung:**  
```sh
blender --background --python scripts/root_obj_to_gltf.py
```
- Konvertiert alle `.obj`-Dateien aus `generated/generate_roots/` automatisch zu `.glb`-Dateien und legt sie in `assets/models/roots/` ab.

---

## 🌳 Baum-OBJ zu GLB konvertieren

**Script:** `batch_trees_obj_to_gltf.py`

**Voraussetzungen:**  
- [Blender](https://www.blender.org/download/) (empfohlen: offizielle Version, nicht Flatpak/Snap)

**Verwendung:**  
```sh
blender --background --python scripts/batch_trees_obj_to_gltf.py
```
- Konvertiert alle `.obj`-Dateien aus `generated/generate_trees/` automatisch zu `.glb`-Dateien und legt sie in `assets/models/trees/` ab.

---

## 💡 Hinweise & Tipps

- Die Collider-JSON-Dateien sorgen dafür, dass die Bäume in Bevy exakt und performant kollidieren – sogar mit separatem Collider für Stamm und Krone!
- Die Musik ist garantiert copyright-frei.
- Für maximale Performance werden Collider im Spiel nur in der Nähe des Spielers gespawnt (Collider-Culling).
- Du kannst die Skripte beliebig anpassen und erweitern – lass deiner Kreativität freien Lauf!

---

## 🤖 Generierungshinweis

> **Hinweis:**  
> Der Großteil dieses Readmes sowie der Skripte und ein erheblicher Teil des Codes wurden mit Unterstützung von GitHub Copilot (GPT-4.1) unter Anleitung von Relacibo generiert.  
>  
> _Mit ❤️ von GitHub Copilot/GPT-4.1 unter Anweisung von Relacibo._
