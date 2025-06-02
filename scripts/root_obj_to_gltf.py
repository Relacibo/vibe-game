import bpy
import os

script_dir = os.path.dirname(os.path.abspath(__file__))
input_dir = os.path.abspath(os.path.join(script_dir, "..", "generated", "generate_roots"))
output_dir = os.path.abspath(os.path.join(script_dir, "..", "assets", "models", "roots"))
os.makedirs(output_dir, exist_ok=True)

for filename in os.listdir(input_dir):
    if filename.lower().endswith(".obj"):
        bpy.ops.wm.read_factory_settings(use_empty=True)
        obj_path = os.path.join(input_dir, filename)
        glb_path = os.path.join(output_dir, filename.replace(".obj", ".glb"))
        bpy.ops.wm.obj_import(filepath=obj_path)
        # Braunes Material erstellen
        mat = bpy.data.materials.new(name="RootBrown")
        mat.diffuse_color = (0.35, 0.18, 0.08, 1.0)  # RGBA, Braun
        for obj in bpy.context.selected_objects:
            if obj.type == 'MESH':
                # Material zuweisen
                if obj.data.materials:
                    obj.data.materials[0] = mat
                else:
                    obj.data.materials.append(mat)
                bpy.context.view_layer.objects.active = obj
                bpy.ops.object.shade_smooth()
        bpy.ops.export_scene.gltf(filepath=glb_path, export_format='GLB', export_yup=False)
        print(f"Exportiert: {glb_path}")
