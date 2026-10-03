# Blender reads back what phiber's Dope Sheet saved (`tests/dope_sheet_edits_keys.rs`
# keeps each save as phiber_dope_keys_<name>.blend in the temp folder):
#   moved   - the Cube's keys at 1 and 20, the one at 20 dragged to 30;
#   deleted - the keys at 1 deleted, the one at 20 alone;
#   emptied - every key deleted, CubeAction left without curves.
# For each, every CubeAction curve's keys and handles as Blender reads them,
# then the handles Blender works out itself (`FCurve.update()`: sort and
# `BKE_fcurve_handles_recalc`) — they must not move.
#   python3 blender_dope_keys.py [temp folder]
import bpy, sys, os, tempfile

folder = sys.argv[-1] if len(sys.argv) > 1 and os.path.isdir(sys.argv[-1]) else tempfile.gettempdir()
worst = 0.0
for name in ("moved", "deleted", "emptied"):
    path = os.path.join(folder, f"phiber_dope_keys_{name}.blend")
    bpy.ops.wm.open_mainfile(filepath=path)
    action = bpy.data.actions["CubeAction"]
    curves = [c for bag in (s.channelbags for layer in action.layers for s in layer.strips) for b in bag for c in b.fcurves]
    print(f"{name}: {len(curves)} curve(s)")
    for c in curves:
        keys = [(tuple(k.co), tuple(k.handle_left), tuple(k.handle_right)) for k in c.keyframe_points]
        print(f"  {c.data_path}[{c.array_index}]", [(round(co[0], 4), round(co[1], 4)) for co, _, _ in keys])
        c.update()
        for (co, h1, h2), k in zip(keys, c.keyframe_points):
            worst = max(worst, *(abs(a - b) for a, b in zip(h1 + h2, tuple(k.handle_left) + tuple(k.handle_right))))
    cube = bpy.data.objects["Cube"]
    bpy.context.scene.frame_set(25)
    print(f"  Cube x at 25: {cube.location.x:.4f}")
print(f"worst handle Blender moved: {worst:.2e}")
