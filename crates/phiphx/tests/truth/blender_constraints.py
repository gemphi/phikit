# Makes `fixtures/constraints.blend` and `truth/constraints.txt` (Blender
# 5.0): one rig with a bone under each constraint batch 89 ports — Child Of
# (whole and filtered), Track To, Locked Track, Limit Location, Limit
# Scale, Maintain Volume, Transformation, Floor, Action and Pivot — driven
# by two animated control bones and an animated Empty, and every bone's
# pose matrix at six frames. For `blend/rig/constraint/tests.rs`. Run with
# the bpy module:
#
#   python3 blender_constraints.py ../fixtures/constraints.blend constraints.txt
import bpy, sys, math
from mathutils import Matrix, Vector

out, truth = sys.argv[-2], sys.argv[-1]
bpy.ops.wm.read_homefile(app_template="")
for ob in list(bpy.data.objects):
    bpy.data.objects.remove(ob)
scene = bpy.context.scene
scene.frame_start, scene.frame_end = 1, 40

aim = bpy.data.objects.new("Aim", None)
scene.collection.objects.link(aim)
arm = bpy.data.armatures.new("Rig")
rig = bpy.data.objects.new("Rig", arm)
scene.collection.objects.link(rig)
rig.location, rig.rotation_euler, rig.scale = (0.5, -0.3, 0.2), (0.1, 0.0, 0.3), (1.0, 1.0, 1.0)
bpy.context.view_layer.objects.active = rig
bpy.ops.object.mode_set(mode="EDIT")

BONES = ["ctl", "ctl2", "child_full", "child_part", "track_z", "track_up", "locked_zy", "locked_xz",
         "limit_loc", "limit_scale", "volume", "xf_loc_rot", "xf_rot_loc", "xf_scale", "xf_quat",
         "xf_before", "floor", "floor_rot", "action", "action_time", "pivot", "pivot_axis", "track_empty"]
root = arm.edit_bones.new("root")
root.head, root.tail = (0, 0, 0), (0, 0.5, 0)
for i, name in enumerate(BONES):
    b = arm.edit_bones.new(name)
    x = (i % 6) * 0.8 - 2.0
    z = (i // 6) * 0.9
    b.head, b.tail = (x, 0.2, z), (x, 0.2 + 0.6, z + 0.15)
    b.roll = 0.1 * i
    b.parent = root
bpy.ops.object.mode_set(mode="POSE")
pb = rig.pose.bones


def con(bone, kind, **props):
    c = pb[bone].constraints.new(kind)
    for k, v in props.items():
        setattr(c, k, v)
    return c


def target(c, sub="ctl", ob=None):
    c.target = ob or rig
    if ob is None:
        c.subtarget = sub


c = con("child_full", "CHILD_OF"); target(c)
c.inverse_matrix = Matrix.Translation((0.2, -0.1, 0.3)) @ Matrix.Rotation(0.4, 4, "Z")
c = con("child_part", "CHILD_OF", use_location_x=False, use_rotation_y=False, use_scale_z=False); target(c, "ctl2")
c.inverse_matrix = Matrix.Rotation(-0.3, 4, "X")
c = con("track_z", "TRACK_TO", track_axis="TRACK_NEGATIVE_Z", up_axis="UP_Y"); target(c)
c = con("track_up", "TRACK_TO", track_axis="TRACK_X", up_axis="UP_Z", use_target_z=True); target(c, "ctl2")
c = con("locked_zy", "LOCKED_TRACK", track_axis="TRACK_Y", lock_axis="LOCK_Z"); target(c)
c = con("locked_xz", "LOCKED_TRACK", track_axis="TRACK_NEGATIVE_Z", lock_axis="LOCK_X"); target(c, "ctl2")
con("limit_loc", "LIMIT_LOCATION", use_min_x=True, min_x=-0.2, use_max_z=True, max_z=0.3, use_max_y=True, max_y=0.1, owner_space="LOCAL")
con("limit_scale", "LIMIT_SCALE", use_min_x=True, min_x=0.8, use_max_y=True, max_y=1.2, use_min_z=True, min_z=0.9, owner_space="LOCAL")
con("volume", "MAINTAIN_VOLUME", mode="UNIFORM", free_axis="SAMEVOL_Y", volume=1.3, owner_space="LOCAL")
c = con("xf_loc_rot", "TRANSFORM", map_from="LOCATION", map_to="ROTATION", mix_mode_rot="ADD", owner_space="LOCAL", target_space="LOCAL")
target(c); c.from_min_x, c.from_max_x, c.to_min_x_rot, c.to_max_x_rot = -1.0, 1.0, -0.8, 0.8
c.map_to_y_from, c.from_min_z, c.from_max_z, c.to_min_y_rot, c.to_max_y_rot = "Z", -0.5, 0.5, 0.2, -0.6
c = con("xf_rot_loc", "TRANSFORM", map_from="ROTATION", map_to="LOCATION", mix_mode="REPLACE", use_motion_extrapolate=True, owner_space="LOCAL", target_space="LOCAL")
target(c, "ctl2"); c.from_min_z_rot, c.from_max_z_rot, c.to_min_x, c.to_max_x = -1.0, 1.0, -0.5, 0.5
c.to_euler_order = "ZXY"
c = con("xf_scale", "TRANSFORM", map_from="SCALE", map_to="SCALE", mix_mode_scale="MULTIPLY", owner_space="LOCAL", target_space="LOCAL")
target(c, "ctl2"); c.from_min_y_scale, c.from_max_y_scale, c.to_min_y_scale, c.to_max_y_scale = 0.5, 2.0, 0.7, 1.6
c.from_min_x_scale, c.from_max_x_scale, c.to_min_x_scale, c.to_max_x_scale = 0.5, 2.0, 1.0, 1.0
c.from_min_z_scale, c.from_max_z_scale, c.to_min_z_scale, c.to_max_z_scale = 0.5, 2.0, 1.0, 1.0
c = con("xf_quat", "TRANSFORM", map_from="ROTATION", map_to="ROTATION", from_rotation_mode="QUATERNION", mix_mode_rot="AFTER", owner_space="LOCAL", target_space="LOCAL")
target(c, "ctl2"); c.from_min_x_rot, c.from_max_x_rot, c.to_min_z_rot, c.to_max_z_rot = -1.0, 1.0, -0.5, 0.5
c = con("xf_before", "TRANSFORM", map_from="LOCATION", map_to="ROTATION", mix_mode_rot="BEFORE", to_euler_order="YXZ", owner_space="WORLD", target_space="WORLD")
target(c); c.from_min_y, c.from_max_y, c.to_min_x_rot, c.to_max_x_rot = -1.0, 1.0, -1.2, 1.2
c = con("floor", "FLOOR", floor_location="FLOOR_Z", offset=0.1); target(c)
c = con("floor_rot", "FLOOR", floor_location="FLOOR_NEGATIVE_Y", use_rotation=True, offset=-0.05); target(c, "ctl2")

swing = bpy.data.actions.new("Swing")
rig.animation_data_create()
rig.animation_data.action = swing
for f, ang in ((1, 0.0), (10, 0.9), (20, -0.4)):
    pb["action"].rotation_mode = "XYZ"
    pb["action"].rotation_euler = (ang, 0.3 * ang, 0.0)
    pb["action"].location = (0.0, 0.2 * ang, 0.0)
    pb["action"].keyframe_insert("rotation_euler", frame=f)
    pb["action"].keyframe_insert("location", frame=f)
pb["action"].rotation_euler = (0, 0, 0)
pb["action"].location = (0, 0, 0)
slot = rig.animation_data.action_slot
rig.animation_data.action = None
c = con("action", "ACTION", transform_channel="LOCATION_X", target_space="LOCAL", min=-1.0, max=1.0, frame_start=1, frame_end=20, mix_mode="AFTER_FULL")
target(c); c.action = swing; c.action_slot = slot
c = con("action_time", "ACTION", use_eval_time=True, eval_time=0.35, frame_start=1, frame_end=20, mix_mode="BEFORE_SPLIT")
c.action = swing; c.action_slot = slot
c = con("pivot", "PIVOT", offset=(0.3, 0.0, 0.1)); target(c, "ctl2")
con("pivot_axis", "PIVOT", rotation_range="X", offset=(0.0, 0.4, 0.0), use_relative_location=True)
c = con("track_empty", "TRACK_TO", track_axis="TRACK_Y", up_axis="UP_Z"); target(c, ob=aim)

act = bpy.data.actions.new("Motion")
rig.animation_data.action = act
pb["ctl2"].rotation_mode = "YZX"
KEYS = [
    ("ctl", "location", [(1, (0, 0, 0)), (20, (0.8, -0.6, 0.5)), (40, (-0.7, 0.4, -0.3))]),
    ("ctl", "rotation_quaternion", [(1, (1, 0, 0, 0)), (40, (0.8, 0.3, -0.4, 0.2))]),
    ("ctl2", "rotation_euler", [(1, (0.2, -0.3, 0.1)), (25, (-0.6, 0.9, 1.1)), (40, (0.4, 0.2, -0.8))]),
    ("ctl2", "scale", [(1, (1, 1, 1)), (30, (1.4, 0.6, 1.2))]),
    ("ctl2", "location", [(1, (0, 0, 0)), (40, (0.2, 0.3, -0.4))]),
    ("limit_loc", "location", [(1, (-0.5, 0.4, 0)), (40, (0.3, -0.2, 0.9))]),
    ("limit_scale", "scale", [(1, (0.5, 1.5, 1.0)), (40, (1.2, 0.8, 0.6))]),
    ("volume", "scale", [(1, (1, 1, 1)), (40, (1.3, 0.7, 1.1))]),
    ("floor", "location", [(1, (0, 0, 0)), (40, (0.1, 0, -1.5))]),
    ("floor_rot", "location", [(1, (0, 0.4, 0)), (40, (0, -1.2, 0.2))]),
    ("pivot", "rotation_quaternion", [(1, (1, 0, 0, 0)), (40, (0.7, 0.4, 0.5, -0.3))]),
    ("pivot_axis", "rotation_quaternion", [(1, (0.9, -0.3, 0.2, 0.1)), (40, (0.8, 0.5, -0.1, 0.2))]),
]
for bone, prop, keys in KEYS:
    for f, v in keys:
        setattr(pb[bone], prop, v)
        pb[bone].keyframe_insert(prop, frame=f)
aim.location = (1.5, 2.0, 1.0)
aim.keyframe_insert("location", frame=1)
aim.location = (-1.0, 1.5, 2.5)
aim.keyframe_insert("location", frame=40)
bpy.ops.object.mode_set(mode="OBJECT")
bpy.ops.wm.save_as_mainfile(filepath=out)


def mat(m):
    return " ".join(f"{m[r][c]:.6f}" for c in range(4) for r in range(4))


with open(truth, "w") as f:
    for frame in (1, 7, 13, 20, 28, 35):
        scene.frame_set(frame)
        graph = bpy.context.evaluated_depsgraph_get()
        ev = rig.evaluated_get(graph)
        for b in ev.pose.bones:
            f.write(f"{frame}\tPM\t{b.name}\t{mat(b.matrix)}\n")
        f.write(f"{frame}\tOB\tAim\t{mat(aim.evaluated_get(graph).matrix_world)}\n")
