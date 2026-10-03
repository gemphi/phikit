"""Truth for task 17 (a)+(b): the metarig object's world matrix, and the pose
matrices of every `rig` bone whose whole parent chain is free of constraints
and drivers, at several frames. Written as text: frame, name, 16 floats."""
import bpy, sys
try: bpy.ops.wm.open_mainfile(filepath=sys.argv[-2], load_ui=False)
except RuntimeError: pass
out=open(sys.argv[-1],"w"); sc=bpy.context.scene
rig=bpy.data.objects["rig"]; meta=bpy.data.objects["Character-Metarig"]
driven=set()
for d in (rig.animation_data.drivers if rig.animation_data else []):
    if 'pose.bones["' in d.data_path: driven.add(d.data_path.split('"')[1])
def clean(b):
    while b:
        if b.constraints or b.name in driven: return False
        b=b.parent
    return True
bones=[b.name for b in rig.pose.bones if clean(b)]
print("clean bones", len(bones), bones[:20])
for f in [1,20,40,66,100,132]:
    sc.frame_set(f); bpy.context.view_layer.update()
    m=meta.matrix_world; out.write(f"{f}\tOB\tCharacter-Metarig\t"+" ".join(f"{m[r][c]:.6f}" for c in range(4) for r in range(4))+"\n")
    for n in bones:
        m=rig.pose.bones[n].matrix
        out.write(f"{f}\tPB\t{n}\t"+" ".join(f"{m[r][c]:.6f}" for c in range(4) for r in range(4))+"\n")
    # matrix_basis = the bone's own loc / rot / scale channels as a matrix
    # (`BKE_pchan_to_mat4`): F-curve evaluation without the parent chain or
    # constraints in the way. Every bone, keyed or not.
    # PBD marks a bone with a driver on one of its own channels: its basis
    # is driver output, which is stage (c), not F-curve evaluation.
    for b in rig.pose.bones:
        m=b.matrix_basis; kind="PBD" if b.name in driven else "PBB"
        out.write(f"{f}\t{kind}\t{b.name}\t"+" ".join(f"{m[r][c]:.6f}" for c in range(4) for r in range(4))+"\n")
out.close(); print("ok")
