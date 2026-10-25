"""Builds a `preview` example's dump in Blender and renders it.
python3 tools/preview.py <dump> <out.png> [camera: wide|court|stand|board|figures|back]"""
import bpy, struct, sys, math
import numpy as np

dump, out = sys.argv[-3], sys.argv[-2]
view = sys.argv[-1]
bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
data = open(dump, "rb").read()
at = 0
def take(fmt):
    global at
    v = struct.unpack_from(fmt, data, at)
    at += struct.calcsize(fmt)
    return v
(count,) = take("<I")
reach = 0.0
for _ in range(count):
    (n,) = take("<I"); name = data[at:at + n].decode(); at += n
    (nv,) = take("<I")
    verts = np.frombuffer(data, "<f4", nv * 5, at).reshape(nv, 5); at += nv * 20
    (ni,) = take("<I")
    idx = np.frombuffer(data, "<u4", ni, at).reshape(-1, 3); at += ni * 4
    base = take("<9f")
    w, h = take("<II")
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts[:, :3].tolist(), [], idx.tolist())
    uv = mesh.uv_layers.new()
    uv.data.foreach_set("uv", verts[idx.ravel(), 3:5].ravel())
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (*base[0:3], 1)
    bsdf.inputs["Roughness"].default_value = base[3]
    bsdf.inputs["Metallic"].default_value = base[4]
    bsdf.inputs["Emission Color"].default_value = (*base[5:8], 1)
    bsdf.inputs["Emission Strength"].default_value = base[8]
    if w:
        px = np.frombuffer(data, np.uint8, w * h * 4, at).reshape(h, w, 4)[::-1].astype(np.float32) / 255.0; at += w * h * 4
        img = bpy.data.images.new(name, w, h)
        img.colorspace_settings.name = "sRGB"
        img.pixels.foreach_set(px.ravel())
        tex = mat.node_tree.nodes.new("ShaderNodeTexImage"); tex.image = img
        mat.node_tree.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
        if base[8] > 0:
            mat.node_tree.links.new(tex.outputs["Color"], bsdf.inputs["Emission Color"])
    mesh.materials.append(mat)
    ob = bpy.data.objects.new(name, mesh)
    sc.collection.objects.link(ob)
    if nv:
        reach = max(reach, float(np.abs(verts[:, :2]).max()))
sun = bpy.data.objects.new("Sun", bpy.data.lights.new("Sun", "SUN")); sun.data.energy = 3.0
sun.rotation_euler = (math.radians(45), 0, math.radians(30)); sc.collection.objects.link(sun)
world = bpy.data.worlds.new("W"); world.use_nodes = True
world.node_tree.nodes["Background"].inputs[0].default_value = (0.55, 0.62, 0.72, 1); world.node_tree.nodes["Background"].inputs[1].default_value = 0.8
sc.world = world
cam = bpy.data.objects.new("Cam", bpy.data.cameras.new("Cam")); sc.collection.objects.link(cam); sc.camera = cam
from mathutils import Vector
if view == "wide":
    cam.location = Vector((reach * 1.5, -reach * 1.7, reach * 1.1)); target = Vector((0, 0, 0)); cam.data.lens = 28
elif view == "figures":
    cam.location = Vector((0, -7.5, 1.2)); target = Vector((0, 0, 0.95)); cam.data.lens = 50
elif view == "back":
    cam.location = Vector((0, 7.5, 1.2)); target = Vector((0, 0, 0.95)); cam.data.lens = 50
elif view == "board":
    cam.location = Vector((0, -16, 4)); target = Vector((0, 0, 10.5)); cam.data.lens = 50
elif view == "court":
    cam.location = Vector((0, -reach * 0.45, reach * 0.12)); target = Vector((0, 0, 1)); cam.data.lens = 22
else:
    cam.location = Vector((reach * 0.3, -reach * 0.2, 3.0)); target = Vector((reach * 0.6, -reach * 0.9, 4.0)); cam.data.lens = 24
cam.rotation_euler = (target - cam.location).to_track_quat("-Z", "Y").to_euler()
sc.render.engine = "CYCLES"; sc.cycles.samples = 16; sc.cycles.device = "CPU"
sc.render.resolution_x, sc.render.resolution_y = 640, 360
sc.render.filepath = out
bpy.ops.render.render(write_still=True)
