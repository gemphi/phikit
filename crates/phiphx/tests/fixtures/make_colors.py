# A cube with a byte colour attribute painted a colour per corner — the
# active one — and a plane with a float colour attribute on its points, for
# `tests/colors_truth.rs` (Blender 5.0). Run: blender -b -P make_colors.py --
# colors.blend
import bpy, sys
bpy.ops.wm.read_factory_settings(use_empty=False)
cube = bpy.data.objects["Cube"]
col = cube.data.color_attributes.new("Col", "BYTE_COLOR", "CORNER")
cube.data.color_attributes.active_color = col
for i, c in enumerate(col.data):
    c.color_srgb = (1.0, 0.0, 0.0, 1.0) if i % 3 == 0 else (0.0, 1.0, 0.0, 1.0) if i % 3 == 1 else (0.0, 0.0, 1.0, 1.0)
bpy.ops.mesh.primitive_plane_add(location=(4, 0, 0))
plane = bpy.context.object
plane.name = "Painted"
pc = plane.data.color_attributes.new("Paint", "FLOAT_COLOR", "POINT")
plane.data.color_attributes.active_color = pc
for i, c in enumerate(pc.data):
    c.color = (0.25 * i, 0.5, 1.0 - 0.25 * i, 1.0)
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-1], compress=True)
