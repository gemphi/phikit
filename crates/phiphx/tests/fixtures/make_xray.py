# The factory startup with X-ray on in the Layout viewport — for
# `tests/view_xray.rs` (Blender 5.0): Solid's X-ray on at 0.3, Wireframe's
# off at 0.6, so every one of the four values differs from the factory's.
import bpy, sys
bpy.ops.wm.read_factory_settings(use_empty=False)
for area in bpy.data.screens["Layout"].areas:
    if area.type == "VIEW_3D":
        shading = area.spaces[0].shading
        shading.show_xray, shading.xray_alpha = True, 0.3
        shading.show_xray_wireframe, shading.xray_alpha_wireframe = False, 0.6
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-1], compress=True)
