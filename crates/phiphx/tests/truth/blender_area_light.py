# What one area light **read from a file** puts on the measuring plane —
# Cycles' own number, in scene-linear radiance, for
# `phiber/tests/a_files_area_light_reaches_the_kernel_at_its_watts.rs`
# (`rendering.md` §71).
#
#   blender -b -P blender_area_light.py [-- OUT.txt]
#   python blender_area_light.py [-- OUT.txt]   (pip install bpy)
#
# `blender_lamp_power.py` built its scene in the script so nothing about it
# could drift, and fed the kernel's question that way; this one is its
# complement — the same rig (`fixtures/AREALIGHT.blend`, made by
# `fixtures/make_arealight.py`), but saved as a `.blend` and loaded back,
# because the question left open at batch 173 is what the *file* stores.
# For an area light `Lamp.energy` in the DNA is 4/pi of the watts the UI
# shows (batch 173: exactly 4/pi, independent of shape and size), and the
# kernel wants watts (`phiray/tests/lamp_power.rs` fed it 100 by hand and
# matched Cycles). The RNA value read back here is the UI's watts; what
# phiphx reads from the block is 4/pi of it — the two printed numbers are
# one measurement.
import bpy, sys, os

out = None
if "--" in sys.argv:
    rest = sys.argv[sys.argv.index("--") + 1:]
    out = rest[0] if rest else None

# The fixture, beside this script's folder — opened rather than assumed,
# so the run works the same under `blender -b` (nothing open yet) and as a
# bpy module (nothing open at all).
fixture = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "fixtures", "AREALIGHT.blend"))
bpy.ops.wm.open_mainfile(filepath=fixture)

sc = bpy.context.scene
sc.render.engine = 'CYCLES'
sc.cycles.device = 'CPU'
sc.cycles.samples = 256
# One bounce only: the number must be the lamp's direct light on the
# plane, not the plane lighting itself through the (absent) world.
sc.cycles.max_bounces = 1
sc.cycles.diffuse_bounces = 1
sc.render.resolution_x = sc.render.resolution_y = 32
sc.render.resolution_percentage = 100
sc.view_settings.view_transform = 'Standard'
sc.render.film_transparent = False
if hasattr(sc.render.image_settings, "media_type"):
    sc.render.image_settings.media_type = 'IMAGE'
sc.render.image_settings.file_format = 'OPEN_EXR'
sc.render.image_settings.color_depth = '32'

la = bpy.data.lights["Area"]
f = os.path.abspath(os.path.join(os.path.dirname(__file__), "area_file.exr"))
sc.render.filepath = f
bpy.ops.render.render(write_still=True)
im = bpy.data.images.load(f)
w_, h_ = im.size
px = list(im.pixels)
i = ((h_ // 2) * w_ + (w_ // 2)) * 4
v = px[i:i + 3]
bpy.data.images.remove(im)
os.remove(f)

lines = [
    f"area_file AREA square size=1 power={la.energy:.1f}W (RNA: the watts the UI shows, "
    f"4/pi of what the DNA field stores) height=2.0 albedo=0.8 radiance={v[0]:.6f}",
]
text = "\n".join(lines) + "\n"
print(text)
if out:
    open(out, "w").write(text)
