# A bouncing basketball, for watching object animation play in the Animation
# workspace: a UV sphere dropped from 6 m onto the floor, each bounce two
# thirds the height of the last (Z `location` keys), squashed on every
# contact and stretched on the way up (`scale` keys), rolling forward in X —
# 120 frames at 24 fps, Linear extrapolation off so it comes to rest.
#
#   blender --background --python make_basketball.py -- basketball.blend
#
# Keys are F-curves on the object (no rig), so `phiphx` reads them as
# `SceneAction` channels and `scene::motion` poses the ball per frame.
import bpy, sys

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.frame_start, scene.frame_end = 1, 120
scene.render.fps = 24

# The ball: a sphere one unit across, orange like a basketball.
bpy.ops.mesh.primitive_uv_sphere_add(radius=0.5, segments=32, ring_count=16)
ball = bpy.context.active_object
ball.name = "Basketball"
mat = bpy.data.materials.new("Basketball")
mat.use_nodes = True
mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = (0.85, 0.35, 0.08, 1.0)
mat.node_tree.nodes["Principled BSDF"].inputs["Roughness"].default_value = 0.6
ball.data.materials.append(mat)
bpy.ops.object.shade_smooth()

# The floor, so there is something to bounce on.
bpy.ops.mesh.primitive_plane_add(size=12)
floor = bpy.context.active_object
floor.name = "Floor"

# A camera and a light so a render works too.
bpy.ops.object.camera_add(location=(9.0, -9.0, 4.0), rotation=(1.2, 0.0, 0.785))
scene.camera = bpy.context.active_object
bpy.ops.object.light_add(type="SUN", location=(4.0, -4.0, 8.0))

# Bounces: each lasts fewer frames and rises less than the last.
RADIUS = 0.5
frame, height, x = 1, 6.0, -3.0
bounces = []
while height > 0.15 and frame < scene.frame_end:
    span = max(6, int(24 * (height / 6.0) ** 0.5))
    bounces.append((frame, height, x, span))
    frame += span
    height *= 2.0 / 3.0
    x += 1.0

def key(f, z, sx, sy, sz):
    ball.location = (x_at(f), 0.0, RADIUS + z)
    ball.scale = (sx, sy, sz)
    ball.keyframe_insert("location", frame=f)
    ball.keyframe_insert("scale", frame=f)

def x_at(f):
    # Rolls forward steadily across the whole clip.
    return -3.0 + (f - 1) / (scene.frame_end - 1) * (len(bounces) + 1.0)

for start, height, _, span in bounces:
    contact = start + span
    # Apex (stretched a touch), falling, contact (squashed), rising.
    key(start, height, 0.95, 0.95, 1.1)
    key(contact - 1, 0.05, 1.0, 1.0, 1.0)
    key(contact, 0.0, 1.25, 1.25, 0.7)
    key(contact + 1, 0.05, 1.0, 1.0, 1.0)
key(scene.frame_end, 0.0, 1.0, 1.0, 1.0)

# A ball falls on a curve: auto-clamped keys, so the apexes stay rounded and
# the contacts stay sharp.
for fc in ball.animation_data.action.fcurves:
    for kp in fc.keyframe_points:
        kp.interpolation = "BEZIER"
        kp.handle_left_type = kp.handle_right_type = "AUTO_CLAMPED"
ball.animation_data.action.name = "Bounce"

scene.frame_set(1)
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-1], compress=True)
