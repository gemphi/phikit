"""Per-object masks for a file's saved view: which object owns each pixel.

    python blender_flat_mask.py -- FILE OUT.png view "<viewinv>" DIST

Every mesh gets a flat emission colour keyed to its index, so the render is a
map of *which object is visible where*. That is what a silhouette mask cannot
show: whether a garment is in front of the body it covers.
"""
import sys
import bpy
import mathutils

args = sys.argv[sys.argv.index("--") + 1:]
path, out, mode = args[0], args[1], args[2]
try:
    bpy.ops.wm.open_mainfile(filepath=path, load_ui=False)
except RuntimeError:
    pass
sc = bpy.context.scene
for o in sc.objects:
    for m in o.modifiers:
        if m.type == "SUBSURF":
            m.render_levels = m.levels

meshes = [o for o in sc.objects if o.type == "MESH" and not o.hide_render]
meshes.sort(key=lambda o: o.name)
for i, o in enumerate(meshes):
    mat = bpy.data.materials.new(f"ID{i}")
    mat.use_nodes = True
    nt = mat.node_tree
    nt.nodes.clear()
    em = nt.nodes.new("ShaderNodeEmission")
    # Index encoded in R and G; B marks "a mesh is here".
    em.inputs[0].default_value = (((i + 1) % 256) / 255.0, ((i + 1) // 256) / 255.0, 1.0, 1.0)
    out_node = nt.nodes.new("ShaderNodeOutputMaterial")
    nt.links.new(em.outputs[0], out_node.inputs[0])
    o.data.materials.clear()
    o.data.materials.append(mat)

if mode == "view":
    inv, dist = eval(args[3]), float(args[4])
    cam = bpy.data.objects.new("View", bpy.data.cameras.new("View"))
    cam.data.sensor_fit, cam.data.sensor_width, cam.data.lens = "HORIZONTAL", 36.0, 25.0
    cam.data.clip_start, cam.data.clip_end = 0.01, 10000.0
    sc.collection.objects.link(cam)
    cam.matrix_world = mathutils.Matrix([[inv[c][r] for c in range(4)] for r in range(4)])
    sc.camera = cam

r = sc.render
r.engine, sc.cycles.samples, sc.cycles.use_denoising = "CYCLES", 1, False
r.resolution_x, r.resolution_y, r.film_transparent, r.filter_size = 480, 270, True, 0.01
sc.view_settings.view_transform = "Standard"
r.image_settings.media_type, r.image_settings.file_format, r.image_settings.color_mode = "IMAGE", "PNG", "RGBA"
r.filepath = out
bpy.ops.render.render(write_still=True)
print("objects:", len(meshes))
for i, o in enumerate(meshes):
    print(f"  {i + 1}\t{o.name}")
