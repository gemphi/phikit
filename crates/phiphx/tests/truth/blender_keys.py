# Blender truth for `blend/anim/summary.rs`: every action's distinct keyed
# frames (the Dope Sheet's key columns), curve count and users.
#
#   blender -b FILE --python blender_keys.py -- OUT.txt
#
# One line per action: name, fcurve count, tab-separated users, then the
# sorted distinct frames rounded to a thousandth.
import bpy, sys

out = sys.argv[sys.argv.index("--") + 1]
users = {}
for ob in bpy.data.objects:
    ad = ob.animation_data
    if ad and ad.action:
        users.setdefault(ad.action.name, []).append(ob.name)

def curves(action):
    if hasattr(action, "layers"):
        for layer in action.layers:
            for strip in layer.strips:
                for bag in strip.channelbags:
                    yield from bag.fcurves
    else:
        yield from action.fcurves

MODE = {"AUTO_RAINBOW": "rainbow", "AUTO_RGB": "rgb", "AUTO_YRGB": "yrgb", "CUSTOM": "custom"}
HANDLE = {"FREE": "free", "AUTO": "auto", "VECTOR": "vector", "ALIGNED": "aligned", "AUTO_CLAMPED": "clamped"}

with open(out, "w") as f:
    for ac in bpy.data.actions:
        frames = set()
        count = 0
        state = {"selected": 0, "rainbow": 0, "rgb": 0, "yrgb": 0, "custom": 0, "keysel": 0,
                 "free": 0, "auto": 0, "vector": 0, "aligned": 0, "clamped": 0, "hsel": 0}
        for fc in curves(ac):
            count += 1
            state["selected"] += int(fc.select)
            state[MODE[fc.color_mode]] += 1
            for kp in fc.keyframe_points:
                frames.add(round(kp.co[0], 3))
                state["keysel"] += int(kp.select_control_point)
                for kind in (kp.handle_left_type, kp.handle_right_type):
                    state[HANDLE[kind]] += 1
                state["hsel"] += int(kp.select_left_handle) + int(kp.select_right_handle)
        # name, curve count, users, key columns, then the Graph editor's
        # per-curve state: how many are selected, the colour modes, how
        # many key points (`BezTriple.f2`) are selected, the handle types
        # (`BezTriple.h1 / h2`) and how many handles (`f1 / f3`) are selected.
        f.write("%s\t%d\t%s\t%s\t%s\n" % (
            ac.name, count, ",".join(sorted(users.get(ac.name, []))),
            " ".join("%g" % x for x in sorted(frames)),
            ";".join("%s=%d" % (k, state[k]) for k in ("selected", "rainbow", "rgb", "yrgb", "custom", "keysel", "free", "auto", "vector", "aligned", "clamped", "hsel"))))
