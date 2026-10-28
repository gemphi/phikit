# A small Rigify-like leg — for `rig/leg_tests.rs`, with every bone's pose
# matrix as Blender evaluates it in `truth/rigify_leg.txt` (Blender 5.2):
#   an FK chain (thigh_fk / shin_fk / foot_fk) and an IK chain (MCH-thigh_ik /
#   MCH-shin_ik, two-bone IK with a pole and stretch) blended onto the ORG
#   chain by two Copy Transforms each, the second at the `IK_FK` custom
#   property's influence through a driver (Rigify's limb switch);
#   a foot roll: `heel` under `foot_ik`, `MCH-roll2` copying the heel's local
#   X and clamped to [0, 360°] by a Limit Rotation with the legacy clamp
#   (what Blender's 4.2 versioning gives every older file, and what Rigify
#   relies on to hold a negative roll at zero), `MCH-roll1` copying the heel
#   in pose space, the IK target under it;
#   tweaks: `MCH-shin_tweak` copying ORG-shin's head and damped-tracking its
#   tail, `shin_tweak` under it half-following `heel` in local space;
#   DEF bones copying the tweaks and stretching to the next (Stretch To,
#   volume XZ, rest length 0 so Blender measures it);
#   a Copy Scale with power and a Limit Distance in custom space (on the
#   pole, to the root: a target inside the IK chain would close a cycle), as the
#   same rigs carry.
# The heel rolls back (negative X) at frame 1, forward at frame 10; the foot
# lifts and the IK/FK switch moves between them.
# Usage: python make_rigify_leg.py <out.blend> <truth.txt>
import bpy, sys
from math import pi, radians
bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
arm = bpy.data.armatures.new("Leg")
ob = bpy.data.objects.new("leg", arm)
sc.collection.objects.link(ob)
bpy.context.view_layer.objects.active = ob
bpy.ops.object.mode_set(mode="EDIT")
eb = arm.edit_bones


def bone(name, head, tail, parent=None, connect=False, roll=0.0, deform=False):
    b = eb.new(name)
    b.head, b.tail, b.roll, b.use_deform = head, tail, roll, deform
    if parent:
        b.parent, b.use_connect = eb[parent], connect
    return b


THIGH, KNEE, ANKLE, TOE, HEEL = (0, 0, 1.0), (0, -0.05, 0.5), (0, 0, 0.1), (0, 0.2, 0.0), (0, -0.08, 0.0)
bone("root", (0, 0, 0), (0, 0.3, 0))
bone("props", (0.5, 0, 0), (0.5, 0.2, 0), "root")
bone("thigh_fk", THIGH, KNEE, "root")
bone("shin_fk", KNEE, ANKLE, "thigh_fk", True)
bone("foot_fk", ANKLE, TOE, "shin_fk", True)
bone("MCH-thigh_ik", THIGH, KNEE, "root")
bone("MCH-shin_ik", KNEE, ANKLE, "MCH-thigh_ik", True)
bone("pole", (0, -0.8, 0.5), (0, -0.8, 0.7), "root")
bone("foot_ik", ANKLE, TOE, "root")
bone("heel", HEEL, (0, -0.08, 0.1), "foot_ik")
bone("MCH-roll2", TOE, (0, 0.2, 0.1), "foot_ik")
bone("MCH-roll1", HEEL, (0, -0.08, 0.1), "MCH-roll2")
bone("MCH-ik_target", ANKLE, TOE, "MCH-roll1")
bone("ORG-thigh", THIGH, KNEE, "root")
bone("ORG-shin", KNEE, ANKLE, "ORG-thigh", True)
bone("ORG-foot", ANKLE, TOE, "ORG-shin", True)
bone("MCH-shin_tweak", KNEE, (0, -0.05, 0.6), "root")
bone("shin_tweak", KNEE, (0, -0.05, 0.6), "MCH-shin_tweak")
bone("foot_tweak", ANKLE, (0, 0, 0.2), "ORG-foot")
bone("DEF-thigh", THIGH, KNEE, "root", deform=True)
bone("DEF-shin", KNEE, ANKLE, "DEF-thigh", True, deform=True)
bone("DEF-foot", ANKLE, TOE, "DEF-shin", True, deform=True)
bone("VIS-pole", KNEE, (0, -0.1, 0.5), "root")
bpy.ops.object.mode_set(mode="OBJECT")
pb = ob.pose.bones
for b in pb:
    b.rotation_mode = "XYZ"
for name in ("MCH-thigh_ik", "MCH-shin_ik"):
    pb[name].ik_stretch = 0.1
    pb[name].lock_ik_y = True
pb["MCH-shin_ik"].lock_ik_z = True
pb["props"]["IK_FK"] = 0.0


def con(owner, kind, target=None, **kw):
    c = pb[owner].constraints.new(kind)
    if target:
        c.target, c.subtarget = ob, target
    for k, v in kw.items():
        setattr(c, k, v)
    return c


ik = con("MCH-shin_ik", "IK", "MCH-ik_target", chain_count=2, use_stretch=True)
ik.pole_target, ik.pole_subtarget, ik.pole_angle = ob, "pole", radians(-90)
con("MCH-roll2", "COPY_ROTATION", "heel", owner_space="LOCAL", target_space="LOCAL", use_y=False, use_z=False)
con("MCH-roll2", "LIMIT_ROTATION", owner_space="LOCAL", use_limit_x=True, min_x=0.0, max_x=2 * pi, use_legacy_behavior=True)
con("MCH-roll1", "COPY_ROTATION", "heel", owner_space="POSE", target_space="POSE")
for org, fk, ikb in (("ORG-thigh", "thigh_fk", "MCH-thigh_ik"), ("ORG-shin", "shin_fk", "MCH-shin_ik"), ("ORG-foot", "foot_fk", "MCH-ik_target")):
    con(org, "COPY_TRANSFORMS", fk)
    c = con(org, "COPY_TRANSFORMS", ikb)
    d = c.driver_add("influence").driver
    d.type = "SUM"
    v = d.variables.new()
    v.name, v.type = "var", "SINGLE_PROP"
    v.targets[0].id, v.targets[0].data_path = ob, 'pose.bones["props"]["IK_FK"]'
con("MCH-shin_tweak", "COPY_LOCATION", "ORG-shin")
con("MCH-shin_tweak", "DAMPED_TRACK", "ORG-shin", head_tail=1.0)
con("MCH-shin_tweak", "COPY_SCALE", "root", power=0.5)
con("shin_tweak", "COPY_TRANSFORMS", "heel", owner_space="LOCAL", target_space="LOCAL", influence=0.5)
con("DEF-thigh", "COPY_TRANSFORMS", "ORG-thigh")
con("DEF-thigh", "STRETCH_TO", "shin_tweak", volume="VOLUME_XZX", rest_length=0.0)
con("DEF-shin", "COPY_TRANSFORMS", "shin_tweak")
con("DEF-shin", "STRETCH_TO", "foot_tweak", rest_length=0.0)
con("DEF-foot", "COPY_TRANSFORMS", "ORG-foot")
con("VIS-pole", "COPY_LOCATION", "ORG-shin")
con("VIS-pole", "STRETCH_TO", "pole", rest_length=0.0)
c = con("pole", "LIMIT_DISTANCE", "root", distance=0.9, limit_mode="LIMITDIST_INSIDE", owner_space="CUSTOM", target_space="CUSTOM")
c.space_object, c.space_subtarget = ob, "root"

ob.animation_data_create()
act = bpy.data.actions.new("LegAction")
ob.animation_data.action = act
ob.animation_data.action_slot = act.slots.new(ob.id_type, "leg")


def key(frame, **vals):
    """`vals`: bone → dict of channel → value."""
    for name, chans in vals.items():
        for chan, v in chans.items():
            if chan == "IK_FK":
                pb[name]["IK_FK"] = v
                pb[name].keyframe_insert('["IK_FK"]', frame=frame)
            else:
                setattr(pb[name], chan, v)
                pb[name].keyframe_insert(chan, frame=frame)


key(1, heel={"rotation_euler": (radians(-40), 0, 0)}, foot_ik={"location": (0, 0, 0)}, props={"IK_FK": 0.0},
    thigh_fk={"rotation_euler": (radians(20), 0, radians(5))}, shin_fk={"rotation_euler": (radians(-30), 0, 0)},
    pole={"location": (0, 0, 0)}, root={"scale": (1, 1, 1)})
key(10, heel={"rotation_euler": (radians(35), radians(10), 0)}, foot_ik={"location": (0.1, 0.05, 0.25)}, props={"IK_FK": 0.7},
    thigh_fk={"rotation_euler": (radians(-50), radians(10), 0)}, shin_fk={"rotation_euler": (radians(40), 0, 0)},
    pole={"location": (0.3, 0, 0)}, root={"scale": (1.2, 1.2, 1.2)})
sc.frame_start, sc.frame_end = 1, 10
sc.frame_set(1)
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[-2], compress=True)


def mat(m): return " ".join("%.6f" % m[r][c] for c in range(4) for r in range(4))


with open(sys.argv[-1], "w") as out:
    for f in (1, 4, 7, 10):
        sc.frame_set(f)
        dg = bpy.context.evaluated_depsgraph_get()
        ev = ob.evaluated_get(dg)
        for b in ev.pose.bones:
            out.write("%d\tPM\t%s\t%s\n" % (f, b.name, mat(b.matrix)))
        for d in ob.animation_data.drivers:
            v = ev.path_resolve(d.data_path)
            out.write("%d\tDV\t%s#%d\t%.6f\n" % (f, d.data_path, d.array_index, float(v)))
    out.write("END\n")
