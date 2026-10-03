"""Truth for task 17 (d): THYMUS's rig as Blender 5.0 evaluates it.

At frames 1 / 20 / 40 / 66 / 100 / 132:
  PM  every pose bone's `matrix` (armature space, after constraints and IK);
  DV  every driver's value on the rig (its data path, index);
  MV  at frames 1 and 66, every 25th vertex of each mesh the rig deforms,
      as its Armature modifier leaves it (the modifiers after it switched
      off, so the vertices keep the mesh's own order), object space.
`python3 blender_rig.py -- <THYMUS.blend> <out.txt>` with bpy installed."""
import bpy, sys
args = sys.argv[sys.argv.index("--") + 1:]
try: bpy.ops.wm.open_mainfile(filepath=args[0], load_ui=False)
except RuntimeError: pass
out = open(args[1], "w"); sc = bpy.context.scene
rig = bpy.data.objects["rig"]
meshes = [o for o in bpy.data.objects if o.type == 'MESH' and any(m.type == 'ARMATURE' and m.object == rig for m in o.modifiers)]
for o in meshes:
    k = next(i for i, m in enumerate(o.modifiers) if m.type == 'ARMATURE' and m.object == rig)
    for m in list(o.modifiers)[k + 1:]:
        m.show_viewport = False
def mat(m): return " ".join(f"{m[r][c]:.6f}" for c in range(4) for r in range(4))
for f in [1, 20, 40, 66, 100, 132]:
    sc.frame_set(f)
    dg = bpy.context.evaluated_depsgraph_get()
    ev = rig.evaluated_get(dg)
    for b in ev.pose.bones:
        out.write(f"{f}\tPM\t{b.name}\t{mat(b.matrix)}\n")
    for d in rig.animation_data.drivers:
        v = ev.path_resolve(d.data_path)
        try: v = v[d.array_index]
        except TypeError: pass
        out.write(f"{f}\tDV\t{d.data_path}#{d.array_index}\t{float(v):.6f}\n")
    for o in (meshes if f in (1, 66) else []):
        me = o.evaluated_get(dg).to_mesh()
        vs = me.vertices; co = " ".join(f"{vs[i].co.x:.5f} {vs[i].co.y:.5f} {vs[i].co.z:.5f}" for i in range(0, len(vs), 25))
        out.write(f"{f}\tMV\t{o.name}\t{len(me.vertices)} {co}\n")
        o.evaluated_get(dg).to_mesh_clear()
out.close(); print("ok")
