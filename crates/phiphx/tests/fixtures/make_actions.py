# The factory startup with three actions — for the Action Editor's browse,
# assign, new and unlink in `blender_opens_saves.rs` (Blender 5.0):
# "CubeAction" moves the Cube (frames 1-20, Manual Frame Range 5-30,
# Cyclic), "CameraAction" moves the Camera, and "Spin" turns the Cube but is
# assigned to nothing (a fake user keeps it).
import bpy, sys
bpy.ops.wm.read_factory_settings(use_empty=False)
cube, camera = bpy.data.objects["Cube"], bpy.data.objects["Camera"]
for frame, x in ((1, 0.0), (20, 3.0)):
    cube.location.x = x
    cube.keyframe_insert("location", frame=frame)
cube.location.x = 0.0
act = cube.animation_data.action
act.use_frame_range = True
act.frame_start, act.frame_end = 5, 30
act.use_cyclic = True
for frame, z in ((1, 5.0), (40, 9.0)):
    camera.location.z = z
    camera.keyframe_insert("location", frame=frame)
cube.animation_data.action = None
for frame, angle in ((1, 0.0), (24, 3.14159)):
    cube.rotation_euler.z = angle
    cube.keyframe_insert("rotation_euler", index=2, frame=frame)
spin = cube.animation_data.action
spin.name = "Spin"
spin.use_fake_user = True
cube.rotation_euler.z = 0.0
cube.animation_data.action = act
bpy.context.scene.frame_set(1)
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-1], compress=True)
