# Blender's Material Preview of a .blend, front orthographic, timed — the
# colour reference for `cargo run -p phiber --example color_render`.
#
#   blender -b -P blender_color.py -- FILE OUT.png [eevee|cycles]
#
# Material Preview is EEVEE lit by the built-in studio HDRI (forest.exr) with
# the scene's own lights and world switched off; the background here is the
# flat viewport grey so the two images differ only where the model does.
import bpy, sys, math, time, os
args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[-2:]
path, out = args[0], args[1]
engine = args[2] if len(args) > 2 else "eevee"
t0 = time.perf_counter()
try: bpy.ops.wm.open_mainfile(filepath=path, load_ui=False)
except RuntimeError: pass
t_read = time.perf_counter() - t0
sc = bpy.context.scene

import numpy as np
t0 = time.perf_counter()
dg = bpy.context.evaluated_depsgraph_get(); lo = np.array([1e9]*3); hi = -lo
for o in sc.objects:
    if o.type != 'MESH' or not o.visible_get(): continue
    ev = o.evaluated_get(dg); me = ev.to_mesh(); n = len(me.vertices)
    if n:
        a = np.empty(n*3); me.vertices.foreach_get("co", a); a = a.reshape(-1, 3)
        W = np.array(ev.matrix_world); a = a @ W[:3, :3].T + W[:3, 3]
        lo = np.minimum(lo, a.min(0)); hi = np.maximum(hi, a.max(0))
    ev.to_mesh_clear()
t_eval = time.perf_counter() - t0

height = hi[2] - lo[2]; cz = (lo[2] + hi[2]) / 2; W, H = 720, 1080; aspect = W / H
cam = bpy.data.cameras.new("front"); cam.type = 'ORTHO'; cam.ortho_scale = height * 1.1 / aspect**2
ob = bpy.data.objects.new("front", cam); sc.collection.objects.link(ob)
ob.location = (0, -10, cz); ob.rotation_euler = (math.pi / 2, 0, 0); sc.camera = ob
sc.render.resolution_x = W; sc.render.resolution_y = H; sc.render.resolution_percentage = 100

# Material Preview: studio HDRI, no scene lights, no scene world.
for o in sc.objects:
    if o.type == 'LIGHT': o.hide_render = True
hdri = os.path.join(bpy.utils.system_resource('DATAFILES', path="studiolights/world"), "forest.exr")
w = bpy.data.worlds.new("preview"); sc.world = w
w.use_nodes = True; nt = w.node_tree; nt.nodes.clear()
env = nt.nodes.new("ShaderNodeTexEnvironment"); env.image = bpy.data.images.load(hdri)
bg = nt.nodes.new("ShaderNodeBackground"); flat = nt.nodes.new("ShaderNodeBackground")
flat.inputs["Color"].default_value = (0.048, 0.048, 0.048, 1)   # the viewport grey
path_n = nt.nodes.new("ShaderNodeLightPath"); mix = nt.nodes.new("ShaderNodeMixShader")
outn = nt.nodes.new("ShaderNodeOutputWorld")
nt.links.new(env.outputs["Color"], bg.inputs["Color"])
nt.links.new(path_n.outputs["Is Camera Ray"], mix.inputs[0])
nt.links.new(bg.outputs[0], mix.inputs[1]); nt.links.new(flat.outputs[0], mix.inputs[2])
nt.links.new(mix.outputs[0], outn.inputs[0])

if engine == "cycles":
    sc.render.engine = 'CYCLES'; sc.cycles.samples = 32; sc.cycles.device = 'CPU'
else:
    sc.render.engine = 'BLENDER_EEVEE'
    try: sc.eevee.taa_render_samples = 32
    except AttributeError: pass
# PHIBER_VIEW=Standard|Filmic|AgX overrides the file's view transform, so a
# comparison can separate the tone curve from the lighting.
# PHIBER_OVERRIDE=1: View Layer → Material Override with a white diffuse, so a
# comparison measures the light and nothing else.
if os.environ.get("PHIBER_OVERRIDE"):
    m = bpy.data.materials.new("override"); m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (0.8, 0.8, 0.8, 1); bsdf.inputs["Roughness"].default_value = 0.5
    bsdf.inputs["Metallic"].default_value = 0.0; bsdf.inputs["Specular IOR Level"].default_value = 0.5
    sc.view_layers[0].material_override = m
view = os.environ.get("PHIBER_VIEW")
if view: sc.view_settings.view_transform = view
sc.render.film_transparent = False
# A file saved for animation has its output set to video; a still needs the
# media type switched back first (5.x), or 'PNG' is not in the enum.
if hasattr(sc.render.image_settings, "media_type"): sc.render.image_settings.media_type = 'IMAGE'
sc.render.image_settings.file_format = 'PNG'; sc.render.filepath = os.path.abspath(out)

t0 = time.perf_counter()
bpy.ops.render.render(write_still=True)
t_render = time.perf_counter() - t0
print(f"blender {engine}: read {t_read*1000:.0f} ms, modifiers+scene {t_eval*1000:.0f} ms, "
      f"render {t_render*1000:.0f} ms (32 samples), total {(t_read+t_eval+t_render)*1000:.0f} ms -> {out}")
