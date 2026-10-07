# Dumps Blender's own evaluated lights and world per frame for
# `fixtures/LIGHTDRIVERS.blend` → `truth/light_driver.txt`.
#
#   python -I blender_light_driver.py <in.blend> <out.txt>
#
# One row per frame per light, and one per frame for the world:
#
#   <frame> light <name> <energy> <r> <g> <b> <radius> <spot_size> <spot_blend> <size> <size_y> <angle>
#   <frame> world <name> <color r g b> <bg strength> <bg color r g b>
#
# A property the lamp type does not have is written as `-`, so the reader is
# held only to what Blender actually keeps for that type.
import bpy, sys

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
blend, out = args[0], args[1]
bpy.ops.wm.open_mainfile(filepath=blend)
sc = bpy.context.scene
FRAMES = [1.0, 6.0, 11.0, 16.0, 21.0]

PROPS = ["energy", "shadow_soft_size", "spot_size", "spot_blend", "size", "size_y", "angle"]

lines = []
for frame in FRAMES:
    sc.frame_set(int(frame))
    dg = bpy.context.evaluated_depsgraph_get()
    for light in sorted(bpy.data.lights, key=lambda d: d.name):
        e = light.evaluated_get(dg)
        cells = [f"{e.color[c]:.6f}" for c in range(3)]
        for name in PROPS:
            cells.append(f"{getattr(e, name):.6f}" if hasattr(e, name) else "-")
        lines.append(f"{frame:g} light {light.name} " + " ".join(cells))
    for world in sorted(bpy.data.worlds, key=lambda d: d.name):
        e = world.evaluated_get(dg)
        cells = [f"{e.color[c]:.6f}" for c in range(3)]
        bg = e.node_tree.nodes.get("Background") if e.node_tree else None
        if bg is None:
            continue
        cells.append(f"{bg.inputs[1].default_value:.6f}")
        cells += [f"{bg.inputs[0].default_value[c]:.6f}" for c in range(3)]
        lines.append(f"{frame:g} world {world.name} " + " ".join(cells))

with open(out, "w") as fh:
    fh.write("\n".join(lines) + "\n")
print(f"wrote {out}: {len(lines)} rows")
