# The factory startup with two more collections beside "Collection" —
# "Props" and "Lamps", the Light moved into "Lamps" — for the Outliner's
# collection drops in `blender_opens_saves.rs` (Blender 5.0).
import bpy, sys
bpy.ops.wm.read_factory_settings(use_empty=False)
scene = bpy.context.scene
for name in ("Props", "Lamps"):
    scene.collection.children.link(bpy.data.collections.new(name))
light = bpy.data.objects["Light"]
for c in list(light.users_collection):
    c.objects.unlink(light)
bpy.data.collections["Lamps"].objects.link(light)
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-1], compress=True)
