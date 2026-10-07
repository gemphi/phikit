# Dumps Blender's own evaluated modifier settings per frame for
# `fixtures/MODDRIVERS.blend` → `truth/mod_driver.txt`.
#
#   python -I blender_mod_driver.py <in.blend> <out.txt>
#
# One row per driven setting per frame, taken from the EVALUATED object:
#
#   <frame> <object> <modifier> <modifier type> <property> <component> <value>
#
# Integers print as integers so the truncation is visible in the sheet.
import bpy, sys

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
blend, out = args[0], args[1]
bpy.ops.wm.open_mainfile(filepath=blend)
sc = bpy.context.scene
FRAMES = [1.0, 6.0, 11.0, 16.0, 21.0]

# Which (modifier, property, component) each object has a driver on.
driven = {}
for ob in bpy.data.objects:
    rows = []
    for fc in (ob.animation_data.drivers if ob.animation_data else []):
        path = fc.data_path
        if not path.startswith('modifiers["'):
            continue
        name = path[len('modifiers["'):path.index('"]', 11)]
        prop = path[path.index('"].') + 3:]
        rows.append((name, prop, fc.array_index))
    if rows:
        driven[ob.name] = sorted(rows)

lines = []
for frame in FRAMES:
    sc.frame_set(int(frame))
    dg = bpy.context.evaluated_depsgraph_get()
    for name in sorted(driven):
        ob = bpy.data.objects[name].evaluated_get(dg)
        for mod_name, prop, comp in driven[name]:
            mod = ob.modifiers.get(mod_name)
            if mod is None:
                continue
            value = getattr(mod, prop)
            value = value[comp] if hasattr(value, "__len__") else value
            kind = mod.bl_rna.properties[prop].type
            cell = f"{int(value)}" if kind == "INT" else f"{value:.6f}"
            lines.append(f"{frame:g} {name} {mod_name} {mod.type} {prop} {comp} {cell}")

with open(out, "w") as fh:
    fh.write("\n".join(lines) + "\n")
print(f"wrote {out}: {len(lines)} rows")
