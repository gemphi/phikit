"""Blender's own render of a .blend from the orthographic front view, with the
file's world and lights left exactly as saved — the reference `color_render`
is measured against.

    python3 blender_world.py FILE OUT.png [samples] [cycles|eevee]

Run it with Blender's Python (`blender --background --python`) or with the
`bpy` module. It prints the read and render times the same way phiber does.
"""

import math
import sys
import time

import bpy
from mathutils import Vector

argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else sys.argv[1:]
path, out = argv[0], argv[1]
samples = int(argv[2]) if len(argv) > 2 else 64
engine = argv[3] if len(argv) > 3 else "cycles"

W, H = 720, 1080

read_start = time.time()
bpy.ops.wm.open_mainfile(filepath=path)
read = time.time() - read_start

scene = bpy.context.scene

# Frame every visible mesh the way `color_render` does: an orthographic front
# view (+X right, +Z up, looking down +Y) sized to the scene's height.
lo = [float("inf")] * 3
hi = [float("-inf")] * 3
for ob in scene.objects:
    if ob.type != "MESH" or not ob.visible_get():
        continue
    for corner in ob.bound_box:
        p = ob.matrix_world @ Vector(corner)
        for k in range(3):
            lo[k] = min(lo[k], p[k])
            hi[k] = max(hi[k], p[k])

height = hi[2] - lo[2]
cam_data = bpy.data.cameras.new("PhiberFront")
cam_data.type = "ORTHO"
cam_data.ortho_scale = height * 1.1
cam = bpy.data.objects.new("PhiberFront", cam_data)
scene.collection.objects.link(cam)
cam.location = (0.0, lo[1] - height * 4.0, (lo[2] + hi[2]) * 0.5)
cam.rotation_euler = (math.pi / 2, 0.0, 0.0)
scene.camera = cam

scene.render.resolution_x = W
scene.render.resolution_y = H
scene.render.resolution_percentage = 100
scene.render.filepath = out
scene.render.image_settings.file_format = "PNG"
# The world is the point of the comparison, so it is not cut out of the film.
scene.render.film_transparent = False

if engine == "cycles":
    scene.render.engine = "CYCLES"
    scene.cycles.device = "CPU"
    scene.cycles.samples = samples
    scene.cycles.use_denoising = False
    scene.cycles.use_adaptive_sampling = False
else:
    scene.render.engine = "BLENDER_EEVEE_NEXT"
    scene.eevee.taa_render_samples = samples

render_start = time.time()
bpy.ops.render.render(write_still=True)
render = time.time() - render_start

print(
    f"blender {engine}: read {read * 1000:.0f} ms, "
    f"render {render * 1000:.0f} ms ({samples} samples), "
    f"total {(read + render) * 1000:.0f} ms -> {out}"
)
