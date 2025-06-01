import bpy
import os

base_folder = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'assets', 'models', 'trees'))
input_folder = base_folder
output_folder = base_folder

for i in range(12):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    trunk_path = os.path.join(input_folder, f"tree_{i}_trunk.obj")
    crown_path = os.path.join(input_folder, f"tree_{i}_crown.obj")

    # Importiere Stamm und Krone
    if os.path.exists(trunk_path):
        bpy.ops.wm.obj_import(filepath=trunk_path)
    if os.path.exists(crown_path):
        bpy.ops.wm.obj_import(filepath=crown_path)

    # Texturen zuweisen (optional, falls nicht automatisch erkannt)
    for obj in bpy.context.selected_objects:
        bpy.context.view_layer.objects.active = obj
        mat = bpy.data.materials.new(name=f"mat_{obj.name}")
        obj.data.materials.append(mat)
        mat.use_nodes = True
        bsdf = mat.node_tree.nodes.get("Principled BSDF")
        tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
        if "trunk" in obj.name.lower():
            tex.image = bpy.data.images.load(os.path.join(input_folder, f"tree_{i}_trunk.png"))
        else:
            tex.image = bpy.data.images.load(os.path.join(input_folder, f"tree_{i}_crown.png"))
        mat.node_tree.links.new(bsdf.inputs['Base Color'], tex.outputs['Color'])

    # Exportiere als GLB (Z-Up, Texturen eingebettet)
    glb_path = os.path.join(output_folder, f"tree_{i}.glb")
    bpy.ops.export_scene.gltf(filepath=glb_path, export_format='GLB', export_yup=False, export_embed_images=True)
    print(f"Exportiert: tree_{i}.glb")