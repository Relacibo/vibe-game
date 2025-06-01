# Scripts

## Übersicht

In diesem Ordner findest du Skripte zur automatischen Generierung und Konvertierung von Baum-Modellen und Texturen (inkl. Bumpmaps/Normalmaps) für das Spiel.

---

## 1. Bäume und Texturen generieren

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
- Erstellt 12 verschiedene Bäume mit Varianz (Stammdicke, Kronengröße, Äste, mehrere Kronen).
- Speichert für jeden Baum:
  - Stamm (`tree_X_trunk.obj`)
  - Kronen (`tree_X_crown0.obj`, ...)
  - Äste (`tree_X_branch0.obj`, ...)
  - Texturen für Stamm/Äste (`tree_X_trunk.png`, `tree_X_branch.png`) und Kronen (`tree_X_crown.png`)
  - **Bumpmap/Normalmap für Stamm/Äste** (`tree_X_trunk_bump.png`)
  - **Bumpmap/Normalmap für Kronen** (`tree_X_crown_bump.png`, simuliert Löcher und Struktur im Blätterdach)
- Alle Dateien werden im Verzeichnis `../assets/models/trees/` abgelegt.

---

## 2. OBJ-Bäume zu GLB konvertieren

**Script:** `batch_obj_to_gltf.py`

**Voraussetzungen:**  
- [Blender](https://www.blender.org/download/) (empfohlen: offizielle Version, nicht Flatpak/Snap)

**Verwendung:**  

**Variante 1: Blender im Hintergrund (empfohlen für Automatisierung)**
```sh
blender --background --python batch_obj_to_gltf.py
```

**Variante 2: Blender-GUI**
1. Starte Blender.
2. Öffne das Scripting-Tab.
3. Lade `batch_obj_to_gltf.py` in den Texteditor.
4. Klicke auf „Run Script“ oder drücke `Alt+P`.

- Beide Varianten importieren alle Baumteile (Stamm, Kronen, Äste), drehen sie korrekt, wenden „Shade Smooth“ an, weisen die passenden Texturen und Bumpmaps zu (inkl. Kronen-Bumpmap) und exportieren für jeden Baum eine GLB-Datei (`tree_X.glb`) mit eingebetteten Texturen und Normalmaps.

---

## Hinweise

- Die generierten GLB-Dateien können direkt in Bevy als Scene geladen werden. Bevy unterstützt Normalmaps aus GLTF/GLB automatisch.
- Für Varianz und Korrektheit der Modelle/Texturen ggf. die Parameter in `generate_trees.py` anpassen.
- Bei Problemen mit Blender-Addons immer die offizielle Blender-Version verwenden.
- Die Skripte sind für Blender 4.4+ angepasst (kein `use_auto_smooth` mehr, stattdessen nur Shade Smooth und konsistente Normalen).

---
