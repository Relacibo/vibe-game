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
        for obj in bpy.context.selected_objects:
            if obj.type == 'MESH':
                bpy.context.view_layer.objects.active = obj
                bpy.ops.object.shade_smooth()
        bpy.ops.export_scene.gltf(filepath=glb_path, export_format='GLB', export_yup=False)
        print(f"Exportiert: {glb_path}")
