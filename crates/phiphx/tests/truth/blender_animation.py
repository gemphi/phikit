# Blender's render of a range of an animation, through the scene's own
# camera at the file's resolution — the reference for
# `cargo run -p phiber --example animation_render`.
#
#   blender -b -P blender_animation.py -- FILE OUTDIR [start] [end] [step] [eevee|cycles|workbench] [samples] [scale]
#
# One PNG per frame, named `f0001.png` as the phiber side names them, plus a
# `times.txt` of `frame posed_ms render_ms samples` so the two can be
# compared on speed as well as on pixels. Nothing about the scene is
# changed: its camera, its lights, its world and its view transform are the
# file's, because the point is what the file looks like in both.
import bpy, sys, os, time

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
path, outdir = args[0], args[1]
start = float(args[2]) if len(args) > 2 else 0.0
end = float(args[3]) if len(args) > 3 else 0.0
step = float(args[4]) if len(args) > 4 else 1.0
engine = args[5] if len(args) > 5 else "eevee"
samples = int(args[6]) if len(args) > 6 else 32
scale = int(args[7]) if len(args) > 7 else 1
os.makedirs(outdir, exist_ok=True)

t0 = time.perf_counter()
try:
    bpy.ops.wm.open_mainfile(filepath=path, load_ui=False)
except RuntimeError:
    pass
t_read = time.perf_counter() - t0
sc = bpy.context.scene

if start <= 0:
    start = sc.frame_start
if end <= 0:
    end = sc.frame_end

if engine == "cycles":
    sc.render.engine = 'CYCLES'; sc.cycles.samples = samples; sc.cycles.device = 'CPU'
elif engine == "workbench":
    sc.render.engine = 'BLENDER_WORKBENCH'
else:
    sc.render.engine = 'BLENDER_EEVEE'
    try:
        sc.eevee.taa_render_samples = samples
    except AttributeError:
        pass

sc.render.resolution_percentage = max(1, 100 // max(1, scale))
# A file saved for animation has its output set to video; a PNG per frame
# needs the media type switched back first (5.x).
if hasattr(sc.render.image_settings, "media_type"):
    sc.render.image_settings.media_type = 'IMAGE'
sc.render.image_settings.file_format = 'PNG'
sc.render.film_transparent = sc.render.film_transparent

# Isolating where the light comes from, for a side-by-side against the
# phiber example's switches of the same names (`rendering.md` §53):
#   PHIBER_NO_WORLD=1   the world contributes nothing (lamps only)
#   PHIBER_NO_LIGHTS=1  every lamp is hidden (the world only)
#   PHIBER_VIEW=Standard|Filmic|AgX   overrides the file's view transform
#   PHIBER_OVERRIDE=<albedo>   View Layer > Override > Material
#                   (`ViewLayer.mat_override`): one grey Lambertian surface
#                   in place of every material in the layer, so the render
#                   measures the light and not the materials.
ovr = os.environ.get("PHIBER_OVERRIDE")
if ovr:
    a = float(ovr)
    mat = bpy.data.materials.new("phiber_override")
    mat.use_nodes = True
    bsdf = next(n for n in mat.node_tree.nodes if n.bl_idname == "ShaderNodeBsdfPrincipled")
    bsdf.inputs["Base Color"].default_value = (a, a, a, 1.0)
    bsdf.inputs["Roughness"].default_value = 1.0
    bsdf.inputs["Metallic"].default_value = 0.0
    bpy.context.view_layer.material_override = mat
if os.environ.get("PHIBER_NO_WORLD") and sc.world:
    for n in sc.world.node_tree.nodes:
        if n.bl_idname == "ShaderNodeBackground":
            n.inputs["Strength"].default_value = 0.0
if os.environ.get("PHIBER_NO_LIGHTS"):
    for o in sc.objects:
        if o.type == 'LIGHT':
            o.hide_render = True
view = os.environ.get("PHIBER_VIEW")
if view:
    sc.view_settings.view_transform = view

print(f"blender {engine}: {sc.camera.name if sc.camera else 'no camera'} "
      f"{sc.render.resolution_x}x{sc.render.resolution_y} at {sc.render.resolution_percentage}%, "
      f"frames {start}..{end} step {step}, read {t_read*1000:.0f} ms")

times = []
f = start
while f <= end + 1e-3:
    t0 = time.perf_counter()
    # The frame the scene stands at. `frame_set` runs the depsgraph — rig,
    # constraints, drivers, modifiers — which is the posing the phiber side
    # times separately as well.
    sc.frame_set(int(f), subframe=f - int(f))
    bpy.context.evaluated_depsgraph_get()
    t_pose = time.perf_counter() - t0
    out = os.path.join(outdir, f"f{int(f):04d}.png")
    sc.render.filepath = os.path.abspath(out)
    t0 = time.perf_counter()
    bpy.ops.render.render(write_still=True)
    t_draw = time.perf_counter() - t0
    line = f"{f} {t_pose*1000:.0f} {t_draw*1000:.0f} {samples}"
    print(line)
    times.append(line)
    f += step

with open(os.path.join(outdir, "times.txt"), "w") as fh:
    fh.write("\n".join(times) + "\n")
print(f"-> {outdir}")
