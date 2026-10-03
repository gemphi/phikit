# Makes `fixtures/object_constraints.blend` and
# `truth/object_constraints.txt` (Blender 5.0): objects that follow others —
# a camera with Track To on an animated Empty, an object under each
# constraint kind in world, local and custom spaces (with and without a
# parent), a chain whose target is itself constrained, an animated
# influence, and objects that follow a bone of an animated rig (Child Of,
# Copy Transforms in the bone's local space, the Armature constraint, a
# bone parent, a relative bone parent) — and every object's world matrix at
# six frames. The file is saved at frame 13. For
# `blend/anim/follow_tests.rs`. Run with the bpy module:
#
#   python3 blender_object_constraints.py ../fixtures/object_constraints.blend object_constraints.txt
import bpy, sys
from mathutils import Matrix

out, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_homefile(app_template="")
for ob in list(bpy.data.objects):
    bpy.data.objects.remove(ob)
scene = bpy.context.scene
scene.frame_start, scene.frame_end = 1, 40


def empty(name, loc=(0, 0, 0), parent=None, inverse=None, mode=None, rot=None, scale=None):
    o = bpy.data.objects.new(name, None)
    scene.collection.objects.link(o)
    if mode:
        o.rotation_mode = mode
    o.location = loc
    if rot is not None:
        if o.rotation_mode == "QUATERNION":
            o.rotation_quaternion = rot
        else:
            o.rotation_euler = rot
    if scale:
        o.scale = scale
    if parent:
        o.parent = parent
        o.matrix_parent_inverse = inverse or Matrix.Identity(4)
    return o


def key(o, prop, keys):
    for f, v in keys:
        setattr(o, prop, v)
        o.keyframe_insert(prop, frame=f)


def con(o, kind, target=None, sub="", **props):
    c = o.constraints.new(kind)
    if target is not None:
        c.target = target
    if sub:
        c.subtarget = sub
    for k, v in props.items():
        setattr(c, k, v)
    return c


# The chain is made first, so its target (the camera) comes after it.
chain = empty("Chain", (0.3, 0.3, 0.3), scale=(0.5, 0.5, 0.5))

aim = empty("Aim", mode="ZYX")
key(aim, "location", [(1, (1.5, 2.0, 1.0)), (40, (-1.0, 1.5, 2.5))])
key(aim, "rotation_euler", [(1, (0, 0, 0)), (40, (0.5, -0.3, 1.2))])
hub = empty("Hub", (0.5, -1.0, 0.3), mode="ZXY", scale=(1.2, 0.8, 1.0))
key(hub, "rotation_euler", [(1, (0.1, 0, 0)), (40, (0.3, -0.2, 1.5))])
kid = empty("Kid", parent=hub, inverse=Matrix.Translation((0.1, 0.2, 0)) @ Matrix.Rotation(0.2, 4, "X"))
key(kid, "location", [(1, (1, 0, 0)), (40, (0.3, 1.0, 0.5))])
key(kid, "rotation_euler", [(1, (0.2, 0, 0.1)), (40, (-0.4, 0.5, 0.9))])

cam_data = bpy.data.cameras.new("Cam")
cam = bpy.data.objects.new("Cam", cam_data)
scene.collection.objects.link(cam)
cam.location = (4, -4, 3)
con(cam, "TRACK_TO", aim, track_axis="TRACK_NEGATIVE_Z", up_axis="UP_Y")
con(cam, "LIMIT_LOCATION", use_max_z=True, max_z=2.5)
key(cam, "location", [(1, (4, -4, 3)), (40, (3, -5, 2))])
scene.camera = cam
con(chain, "COPY_TRANSFORMS", cam)

o = empty("Damp", (-2, 1, 0))
con(o, "DAMPED_TRACK", kid, track_axis="TRACK_Y")
o = empty("CopyLoc", (0.2, 0.1, -0.3), parent=hub)
con(o, "COPY_LOCATION", aim, use_offset=True, target_space="LOCAL", owner_space="LOCAL")
o = empty("CopyRot", (1, -1, 0), mode="YZX", rot=(0.3, 0.2, -0.1))
con(o, "COPY_ROTATION", kid, target_space="LOCAL", owner_space="LOCAL", mix_mode="ADD")
o = empty("CopyScale", (0, 2, 0), scale=(1.1, 0.9, 1.3))
con(o, "COPY_SCALE", hub, power=0.5, use_make_uniform=False)
o = empty("ChildAll", (1, 1, 1))
c = con(o, "CHILD_OF", hub)
c.inverse_matrix = Matrix.Translation((-0.3, 0.2, 0.1)) @ Matrix.Rotation(-0.4, 4, "Z")
o = empty("ChildPart", (0.5, 0.2, -0.4))
c = con(o, "CHILD_OF", aim, use_location_y=False, use_rotation_x=False, use_scale_z=False)
c.inverse_matrix = Matrix.Rotation(0.3, 4, "Y")
o = empty("LimLoc", parent=hub)
key(o, "location", [(1, (-0.6, 0.1, 0.5)), (40, (0.4, 0.9, -0.1))])
con(o, "LIMIT_LOCATION", owner_space="LOCAL", use_min_x=True, min_x=-0.3, use_max_y=True, max_y=0.4, use_max_z=True, max_z=0.2)
o = empty("LimRot", (0, 0, 1), mode="XZY")
key(o, "rotation_euler", [(1, (-1.0, 0.2, 0.3)), (40, (1.2, -0.5, 0.8))])
con(o, "LIMIT_ROTATION", use_limit_x=True, min_x=-0.4, max_x=0.6, use_limit_z=True, min_z=0.0, max_z=0.5)
o = empty("Floor")
key(o, "location", [(1, (1.4, 2.1, 3.0)), (40, (-0.8, 1.2, 0.0))])
con(o, "FLOOR", aim, floor_location="FLOOR_Z", offset=0.2)
o = empty("Locked", (-1, -1, 0.5), rot=(0.2, 0.1, 0.4))
con(o, "LOCKED_TRACK", aim, track_axis="TRACK_Y", lock_axis="LOCK_Z")
o = empty("Xform", (0.1, 0.2, 0.3), parent=hub)
c = con(o, "TRANSFORM", kid, map_from="LOCATION", map_to="ROTATION", mix_mode_rot="ADD", target_space="LOCAL", owner_space="LOCAL")
c.from_min_x, c.from_max_x, c.to_min_z_rot, c.to_max_z_rot = 0.0, 1.0, -0.5, 0.7
o = empty("Pivot", (0.5, 0.5, 0.0), mode="QUATERNION")
key(o, "rotation_quaternion", [(1, (1, 0, 0, 0)), (40, (0.7, 0.4, 0.5, -0.3))])
con(o, "PIVOT", aim, offset=(0.2, 0.0, 0.1), rotation_range="ALWAYS_ACTIVE")
o = empty("Stretch", (-1.5, 0.0, 0.2))
con(o, "STRETCH_TO", aim, rest_length=2.0, volume="VOLUME_XZX")
o = empty("Dist")
key(o, "location", [(1, (0, 0, 0)), (40, (-3, 3, 1))])
con(o, "LIMIT_DISTANCE", aim, distance=2.0, limit_mode="LIMITDIST_INSIDE")
o = empty("Custom", (0.3, -0.2, 0.4))
c = con(o, "COPY_LOCATION", aim, owner_space="CUSTOM", target_space="CUSTOM", use_offset=True)
c.space_object = hub
o = empty("Fade", (2, 0, 0))
c = con(o, "COPY_LOCATION", aim)
c.influence = 0.0
c.keyframe_insert("influence", frame=1)
c.influence = 1.0
c.keyframe_insert("influence", frame=30)

# A rig that moves, and what follows its bones.
arm = bpy.data.armatures.new("Rig")
rig = bpy.data.objects.new("Rig", arm)
scene.collection.objects.link(rig)
rig.location, rig.rotation_euler = (-0.5, 0.8, 0.2), (0.0, 0.2, 0.6)
bpy.context.view_layer.objects.active = rig
bpy.ops.object.mode_set(mode="EDIT")
root = arm.edit_bones.new("root")
root.head, root.tail = (0, 0, 0), (0, 0.4, 0)
b = arm.edit_bones.new("arm")
b.head, b.tail, b.roll, b.parent = (0, 0.4, 0.2), (0.2, 1.2, 0.5), 0.3, root
b.use_relative_parent = True
h = arm.edit_bones.new("hand")
h.head, h.tail, h.parent, h.use_connect = (0.2, 1.2, 0.5), (0.3, 1.6, 0.4), b, True
bpy.ops.object.mode_set(mode="POSE")
pb = rig.pose.bones
con(pb["hand"], "DAMPED_TRACK", aim, track_axis="TRACK_Y").influence = 0.5
rig.animation_data_create()
rig.animation_data.action = bpy.data.actions.new("Wave")
pb["arm"].rotation_mode = "XYZ"
for f, v in ((1, (0, 0, 0)), (20, (0.8, -0.3, 0.4)), (40, (-0.5, 0.6, -0.2))):
    pb["arm"].rotation_euler = v
    pb["arm"].keyframe_insert("rotation_euler", frame=f)
for f, v in ((1, (0, 0, 0)), (40, (0.3, -0.2, 0.5))):
    pb["root"].location = v
    pb["root"].keyframe_insert("location", frame=f)
bpy.ops.object.mode_set(mode="OBJECT")

o = empty("Prop", (0.1, 0.0, 0.2))
con(o, "CHILD_OF", rig, "hand")
o = empty("Held", (0.0, 0.3, 0.0), parent=rig, inverse=Matrix.Translation((0, -0.2, 0.1)))
o.parent_type, o.parent_bone = "BONE", "hand"
o = empty("RelHeld", (0.2, 0.0, 0.0), parent=rig)
o.parent_type, o.parent_bone = "BONE", "arm"
o = empty("Skin", (1.0, 1.0, 0.0))
c = con(o, "ARMATURE")
for name, w in (("arm", 0.6), ("hand", 0.4)):
    t = c.targets.new()
    t.target, t.subtarget, t.weight = rig, name, w
o = empty("BoneLocal", (0.0, 0.0, 1.0))
con(o, "COPY_TRANSFORMS", rig, "arm", target_space="LOCAL")
o = empty("Tail")
con(o, "COPY_LOCATION", rig, "hand", head_tail=0.5)

scene.frame_set(13)
bpy.ops.wm.save_as_mainfile(filepath=out)


def mat(m):
    return " ".join(f"{m[r][c]:.6f}" for c in range(4) for r in range(4))


with open(truth, "w") as f:
    for frame in (1, 7, 13, 20, 28, 35):
        scene.frame_set(frame)
        graph = bpy.context.evaluated_depsgraph_get()
        for o in scene.objects:
            f.write(f"{frame}\tOB\t{o.name}\t{mat(o.evaluated_get(graph).matrix_world)}\n")
