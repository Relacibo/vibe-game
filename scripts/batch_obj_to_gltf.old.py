import bpy
import os

# Zielordner für Input und Output
base_folder = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'assets', 'models', 'trees'))
input_folder = base_folder
output_folder = base_folder

# Finde alle Baum-IDs (angenommen: tree_0_trunk.obj, tree_0_crown.obj, tree_0.obj, ...)
ids = set()
for filename in os.listdir(input_folder):
    if filename.startswith("tree_") and filename.endswith(".obj"):
        parts = filename.split('_')
        if len(parts) >= 2:
            ids.add(parts[1].split('.')[0])

for idx in sorted(ids, key=lambda x: int(x)):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    # Importiere trunk, crown und ggf. den Gesamtbaum (falls vorhanden)
    trunk_path = os.path.join(input_folder, f"tree_{idx}_trunk.obj")
    crown_path = os.path.join(input_folder, f"tree_{idx}_crown.obj")
    obj_path = os.path.join(input_folder, f"tree_{idx}.obj")

    if os.path.exists(trunk_path):
        bpy.ops.wm.obj_import(filepath=trunk_path)
    if os.path.exists(crown_path):
        bpy.ops.wm.obj_import(filepath=crown_path)
    if os.path.exists(obj_path):
        bpy.ops.wm.obj_import(filepath=obj_path)

    # Exportiere alles als eine GLB-Datei
    glb_path = os.path.join(output_folder, f"tree_{idx}.glb")
    bpy.ops.export_scene.gltf(filepath=glb_path, export_format='GLB')
    print(f"Exportiert: tree_{idx}.glb")
