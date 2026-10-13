"""AREALIGHT.blend: one area light of a known power over the measuring
plane, saved as a file so the whole read path is under test. What
`Lamp.energy` stores for an area light is 4/pi of the watts the UI shows
(batch 173: exactly 4/pi, independent of shape and size), and the kernel
wants watts (`phiray/tests/lamp_power.rs` fed it 100 by hand and matched
Cycles) — so what a reader hands the kernel for a *file's* area light is
the quantity this fixture pins.

The rig is `truth/blender_lamp_power.py`'s measuring scene: a 20 x 20 m
plane of albedo 0.8, a black world, an orthographic camera straight down
over a 2 m frame, and one lamp at (0, 0, 2) pointing down — a 100 W
SQUARE area light, 1 m on a side. `truth/blender_area_light.py` renders
this file in Cycles; `phiber/tests/a_files_area_light_reaches_the_kernel_at_its_watts.rs`
carries the number and renders the same file's light through the tracer.

Run with bpy available (pip install bpy), or under `blender -b -P`:
  python make_arealight.py
"""

import bpy

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene

# The measuring plane: Principled Base Color 0.8, Roughness 1, Metallic 0,
# Specular IOR Level 0.5 — the surface `Tri::with_pbr(1.0, 0.0)` is
# (`truth/blender_lamp_power.py`).
bpy.ops.mesh.primitive_plane_add(size=20.0, location=(0, 0, 0))
plane = bpy.context.object
ma = bpy.data.materials.new("grey")
ma.use_nodes = True
diff = ma.node_tree.nodes["Principled BSDF"]
diff.inputs["Base Color"].default_value = (0.8, 0.8, 0.8, 1.0)
diff.inputs["Roughness"].default_value = 1.0
diff.inputs["Metallic"].default_value = 0.0
diff.inputs["Specular IOR Level"].default_value = 0.5
plane.data.materials.append(ma)

# The lamp. `energy = 100` through RNA writes 4/pi of it into the DNA
# field (127.324...), which is exactly the value under test: the file does
# not hold the watts the UI shows.
la = bpy.data.lights.new("Area", type='AREA')
la.energy = 100.0
la.shape = 'SQUARE'
la.size = 1.0
ob = bpy.data.objects.new("Area", la)
sc.collection.objects.link(ob)
ob.location = (0.0, 0.0, 2.0)
ob.rotation_euler = (0.0, 0.0, 0.0)

# A black world: the lamp is the only light.
w = bpy.data.worlds.new("black")
sc.world = w
w.use_nodes = True
bg = w.node_tree.nodes["Background"]
bg.inputs["Color"].default_value = (0, 0, 0, 1)

# The measuring camera: orthographic, straight down, a 2 m frame.
cam = bpy.data.cameras.new("top")
cam.type = 'ORTHO'
cam.ortho_scale = 2.0
cob = bpy.data.objects.new("top", cam)
sc.collection.objects.link(cob)
cob.location = (0.0, 0.0, 10.0)
cob.rotation_euler = (0.0, 0.0, 0.0)
sc.camera = cob

sc.render.resolution_x = sc.render.resolution_y = 32
sc.render.resolution_percentage = 100

bpy.ops.wm.save_as_mainfile(filepath=__file__.replace("make_arealight.py", "AREALIGHT.blend"), compress=True)
