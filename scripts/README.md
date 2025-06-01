# Scripts

## Übersicht

In diesem Ordner findest du Skripte zur automatischen Generierung und Konvertierung von Baum-Modellen und Texturen für das Spiel.

---

## 1. Bäume und Texturen generieren

**Script:** `generate_trees.py`

**Voraussetzungen:**  
- Python 3  
- Pakete: `trimesh`, `numpy`, `Pillow`  
  Installation:  
  ```sh
  pip install trimesh numpy pillow
  ```

**Verwendung:**  
```sh
python3 generate_trees.py
```
- Erstellt 12 verschiedene Bäume als OBJ-Dateien (`tree_X_trunk.obj`, `tree_X_crown.obj`) und passende Texturen (`tree_X_trunk.png`, `tree_X_crown.png`) im Verzeichnis `../assets/models/trees/`.

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

- Beide Varianten exportieren für jeden Baum eine GLB-Datei (`tree_X.glb`) mit eingebetteten Texturen.

---

## Hinweise

- Die generierten GLB-Dateien können direkt in Bevy als Scene geladen werden.
- Für Varianz und Korrektheit der Modelle/Texturen ggf. die Parameter in `generate_trees.py` anpassen.
- Bei Problemen mit Blender-Addons immer die offizielle Blender-Version verwenden.

---
