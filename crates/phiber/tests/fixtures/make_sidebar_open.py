# The factory startup with every 3D Viewport's sidebar (N panel) shown —
# `SpaceView3D.show_region_ui` — for `viewport_sidebar.rs`.
import bpy, sys
bpy.ops.wm.read_factory_settings(use_empty=False)
for screen in bpy.data.screens:
    for area in screen.areas:
        if area.type == 'VIEW_3D':
            area.spaces.active.show_region_ui = True
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-1], compress=True)
