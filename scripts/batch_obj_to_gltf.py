import bpy
import os
import math

base_folder = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'assets', 'models', 'trees'))
input_folder = base_folder
output_folder = base_folder

for i in range(12):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    imported_objects = []

    # Importiere Stamm
    trunk_path = os.path.join(input_folder, f"tree_{i}_trunk.obj")
    if os.path.exists(trunk_path):
        bpy.ops.wm.obj_import(filepath=trunk_path)
        imported_objects += bpy.context.selected_objects

    # Importiere Kronen (beliebig viele)
    j = 0
    while True:
        crown_path = os.path.join(input_folder, f"tree_{i}_crown{j}.obj")
        if not os.path.exists(crown_path):
            break
        bpy.ops.wm.obj_import(filepath=crown_path)
        imported_objects += bpy.context.selected_objects
        j += 1

    # Importiere Äste (beliebig viele)
    j = 0
    while True:
        branch_path = os.path.join(input_folder, f"tree_{i}_branch{j}.obj")
        if not os.path.exists(branch_path):
            break
        bpy.ops.wm.obj_import(filepath=branch_path)
        imported_objects += bpy.context.selected_objects
        j += 1

    # Korrigiere die Ausrichtung: -90° um X und aktiviere Shade Smooth
    for obj in imported_objects:
        obj.rotation_euler[0] = math.radians(-90)
        if obj.type == 'MESH':
            bpy.context.view_layer.objects.active = obj
            bpy.ops.object.shade_smooth()
            bpy.ops.object.mode_set(mode='EDIT')
            bpy.ops.mesh.select_all(action='SELECT')
            bpy.ops.mesh.normals_make_consistent(inside=False)
            bpy.ops.object.mode_set(mode='OBJECT')

    # Texturen zuweisen
    for obj in imported_objects:
        bpy.context.view_layer.objects.active = obj
        mat = bpy.data.materials.new(name=f"mat_{obj.name}")
        obj.data.materials.clear()
        obj.data.materials.append(mat)
        mat.use_nodes = True
        bsdf = mat.node_tree.nodes.get("Principled BSDF")
        tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
        name = obj.name.lower()
        if "trunk" in name or "branch" in name:
            tex.image = bpy.data.images.load(os.path.join(input_folder, f"tree_{i}_trunk.png"))
        elif "crown" in name:
            tex.image = bpy.data.images.load(os.path.join(input_folder, f"tree_{i}_crown.png"))
            # Bumpmap/Normalmap für Krone laden und verbinden
            bump_tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
            bump_tex.image = bpy.data.images.load(os.path.join(input_folder, f"tree_{i}_crown_bump.png"))
            bump_tex.image.colorspace_settings.name = 'Non-Color'
            normal_map = mat.node_tree.nodes.new("ShaderNodeNormalMap")
            mat.node_tree.links.new(normal_map.inputs['Color'], bump_tex.outputs['Color'])
            mat.node_tree.links.new(bsdf.inputs['Normal'], normal_map.outputs['Normal'])
        mat.node_tree.links.new(bsdf.inputs['Base Color'], tex.outputs['Color'])

        # Bumpmap/Normalmap laden und verbinden
        if "trunk" in name or "branch" in name:
            bump_tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
            bump_tex.image = bpy.data.images.load(os.path.join(input_folder, f"tree_{i}_trunk_bump.png"))
            bump_tex.image.colorspace_settings.name = 'Non-Color'
            normal_map = mat.node_tree.nodes.new("ShaderNodeNormalMap")
            mat.node_tree.links.new(normal_map.inputs['Color'], bump_tex.outputs['Color'])
            mat.node_tree.links.new(bsdf.inputs['Normal'], normal_map.outputs['Normal'])

    # Exportiere als GLB (Standard, Texturen werden eingebettet)
    glb_path = os.path.join(output_folder, f"tree_{i}.glb")
    bpy.ops.export_scene.gltf(filepath=glb_path, export_format='GLB', export_yup=False)
    print(f"Exportiere nach: {glb_path}")
