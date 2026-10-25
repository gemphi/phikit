# A two-bone rig posed only by NLA strips — for `rig/nla_tests.rs`, with
# Blender's evaluated bases in `truth/nla_rig.txt` (Blender 5.2):
#   "Lower" track: "Reach" 1-10 (Nothing), "Swing" 10-30 (Hold Forward),
#   back to back as the demo's moves are laid;
#   "Upper" track: "Wave" 40-60 blending in over 6 frames and out over 4,
#   at influence over "Swing" held beneath it;
#   the rig object itself slides with "Slide" on its own track.
# Each bone's saved pose is off its default, so a frame no strip covers
# shows whether evaluation starts from the default, as Blender's does.
# Usage: python make_nla_rig.py <out.blend> <truth.txt>
import bpy, sys
bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
arm = bpy.data.armatures.new("Arm")
ob = bpy.data.objects.new("Rig", arm)
sc.collection.objects.link(ob)
bpy.context.view_layer.objects.active = ob
bpy.ops.object.mode_set(mode="EDIT")
root = arm.edit_bones.new("root"); root.head, root.tail = (0, 0, 0), (0, 0, 1)
tip = arm.edit_bones.new("tip"); tip.head, tip.tail = (0, 0, 1), (0, 0, 2); tip.parent = root
bpy.ops.object.mode_set(mode="OBJECT")
bones = [ob.pose.bones["root"], ob.pose.bones["tip"]]
for pb in bones:
    pb.rotation_mode = "QUATERNION"
ob.animation_data_create()


def action(name, keys):
    """`keys`: frame → (root quaternion, tip quaternion, root location)."""
    act = bpy.data.actions.new(name)
    ob.animation_data.action = act
    for f, (qa, qb, loc) in keys.items():
        bones[0].rotation_quaternion = qa
        bones[1].rotation_quaternion = qb
        bones[0].location = loc
        for pb in bones:
            pb.keyframe_insert("rotation_quaternion", frame=f)
        bones[0].keyframe_insert("location", frame=f)
    return act


reach = action("Reach", {1: ((1, 0, 0, 0), (1, 0, 0, 0), (0, 0, 0)), 10: ((0.8, 0.6, 0, 0), (0.9, 0, 0.44, 0), (0, 0.2, 0))})
swing = action("Swing", {1: ((0.8, 0.6, 0, 0), (0.9, 0, 0.44, 0), (0, 0.2, 0)), 21: ((0.6, -0.8, 0, 0), (1, 0, 0, 0), (0.3, 0, 0.1))})
wave = action("Wave", {1: ((0.7, 0, 0, 0.7), (0.5, 0.5, 0.5, 0.5), (0, 0, 0.5)), 21: ((0.7, 0, 0, -0.7), (0.5, -0.5, 0.5, -0.5), (0, 0, 0.8))})
slide = bpy.data.actions.new("Slide")
ob.animation_data.action = slide
for f, x in ((1, 0.0), (60, 4.0)):
    ob.location.x = x
    ob.keyframe_insert("location", frame=f)
ob.animation_data.action = None

def strip(track, name, start, act, extrapolation, blend=(0, 0)):
    s = track.strips.new(name, start, act)
    s.action_slot = act.slots[0]
    s.extrapolation = extrapolation
    s.blend_in, s.blend_out = blend
    return s

move = ob.animation_data.nla_tracks.new(); move.name = "Slide"
strip(move, "Slide", 1, slide, "HOLD")
lower = ob.animation_data.nla_tracks.new(); lower.name = "Lower"
strip(lower, "Reach", 1, reach, "NOTHING")
strip(lower, "Swing", 10, swing, "HOLD_FORWARD")
upper = ob.animation_data.nla_tracks.new(); upper.name = "Upper"
strip(upper, "Wave", 40, wave, "NOTHING", (6, 4))
# The saved pose, off every default: what a frame no strip covers must not show.
for pb in bones:
    pb.location = (0.5, 0.5, 0.5)
    pb.rotation_quaternion = (0.5, 0.5, 0.5, 0.5)
sc.frame_start, sc.frame_end = 1, 70
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-2], compress=True)
with open(sys.argv[-1], "w") as out:
    for f in (1, 5, 10, 15, 30, 35, 40, 43, 50, 57, 59, 65):
        sc.frame_set(f)
        for pb in bones:
            vals = (*pb.location, *pb.rotation_quaternion)
            out.write(f"{f}\t{pb.name}\t{' '.join(f'{v:.6f}' for v in vals)}\n")
        out.write(f"{f}\tobject\t{ob.location.x:.6f}\n")
