# Dumps Blender's own evaluated material values per frame for
# `fixtures/MATDRIVERS.blend` → `truth/mat_driver.txt`.
#
#   python -I blender_mat_driver.py <in.blend> <out.txt>
#
# The reference is the EVALUATED depsgraph copy of each material, which is
# where a driver's result lands. One row per frame per material:
#
#   <frame> <material> own <roughness> <metallic> <diffuse r g b a> <spec>
#   <frame> <material> sock <node> <input index> <component> <value>
#
# Socket rows are emitted for every input a driver writes, by index, so the
# reader is held to the same index Blender's RNA path names.
import bpy, sys

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
blend, out = args[0], args[1]
bpy.ops.wm.open_mainfile(filepath=blend)
sc = bpy.context.scene
FRAMES = [1.0, 6.0, 11.0, 16.0, 21.0]

# Which (node, input, component) each material's tree has a driver on.
driven = {}
for m in bpy.data.materials:
    rows = []
    nt = m.node_tree
    for fc in (nt.animation_data.drivers if nt and nt.animation_data else []):
        path = fc.data_path
        if not path.startswith('nodes["') or not path.endswith("].default_value"):
            continue
        name = path[len('nodes["'):path.index('"]', 7)]
        index = int(path[path.index(".inputs[") + 8:path.index("].default_value")])
        rows.append((name, index, fc.array_index))
    driven[m.name] = sorted(rows)

lines = []
for frame in FRAMES:
    sc.frame_set(int(frame))
    dg = bpy.context.evaluated_depsgraph_get()
    for m in sorted(bpy.data.materials, key=lambda m: m.name):
        me = m.evaluated_get(dg)
        c = list(me.diffuse_color)
        lines.append(
            f"{frame:g} {m.name} own {me.roughness:.6f} {me.metallic:.6f} "
            f"{c[0]:.6f} {c[1]:.6f} {c[2]:.6f} {c[3]:.6f} {me.specular_intensity:.6f}"
        )
        tree = me.node_tree
        for node, index, comp in driven[m.name]:
            value = tree.nodes[node].inputs[index].default_value
            value = value[comp] if hasattr(value, "__len__") else value
            lines.append(f"{frame:g} {m.name} sock {node} {index} {comp} {value:.6f}")

with open(out, "w") as fh:
    fh.write("\n".join(lines) + "\n")
print(f"wrote {out}: {len(lines)} rows")
