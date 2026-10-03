# Makes `fixtures/nla_cube.blend`: a cube with an active action pushed down
# to two NLA tracks (a clip strip each, one repeated, one muted) and a
# second action left active, plus a sphere with animation data but no
# action — everything the NLA editor lists. Run with the bpy module:
#
#   python3 blender_nla_fixture.py ../fixtures/nla_cube.blend
import bpy, sys

out = sys.argv[1]
bpy.ops.wm.read_homefile(app_template="")
for ob in list(bpy.data.objects):
    bpy.data.objects.remove(ob)

cube_mesh = bpy.data.meshes.new("Cube")
cube_mesh.from_pydata([(-1, -1, -1), (1, -1, -1), (1, 1, -1), (-1, 1, -1), (-1, -1, 1), (1, -1, 1), (1, 1, 1), (-1, 1, 1)], [],
                      [(0, 1, 2, 3), (4, 5, 6, 7), (0, 1, 5, 4), (2, 3, 7, 6), (0, 3, 7, 4), (1, 2, 6, 5)])
cube = bpy.data.objects.new("Cube", cube_mesh)
bpy.context.scene.collection.objects.link(cube)

walk = bpy.data.actions.new("Walk")
cube.animation_data_create()
cube.animation_data.action = walk
if hasattr(walk, "slots"):
    slot = walk.slots.new(cube.id_type, cube.name)
    cube.animation_data.action_slot = slot
for frame, x in ((1, 0.0), (25, 4.0), (49, 0.0)):
    cube.location.x = x
    cube.keyframe_insert("location", index=0, frame=frame)

tracks = cube.animation_data.nla_tracks
lower = tracks.new()
lower.name = "Base Layer"
strip = lower.strips.new("Walk", 1, walk)
strip.repeat = 2.0
upper = tracks.new()
upper.name = "Overlay"
second = upper.strips.new("Walk Again", 120, walk)
second.mute = True
second.select = True
upper.mute = True

idle = bpy.data.actions.new("Idle")
cube.animation_data.action = idle
if hasattr(idle, "slots"):
    slot = idle.slots.new(cube.id_type, cube.name)
    cube.animation_data.action_slot = slot
for frame, z in ((10, 0.0), (30, 1.0)):
    cube.location.z = z
    cube.keyframe_insert("location", index=2, frame=frame)

sphere_mesh = bpy.data.meshes.new("Sphere")
sphere_mesh.from_pydata([(0, 0, 0), (1, 0, 0), (0, 1, 0)], [], [(0, 1, 2)])
sphere = bpy.data.objects.new("Sphere", sphere_mesh)
bpy.context.scene.collection.objects.link(sphere)
sphere.animation_data_create()

bpy.ops.wm.save_as_mainfile(filepath=out, compress=False)
