"""Regenerate tests/truth/nodes.txt with real Blender (pip install bpy):

    python crates/phiphx/tests/truth/blender_nodes.py

For each emission material of NODES.blend (`fixtures/make_nodes.py`), Cycles
bakes the plane's Emit pass to a 4 x 4 float image and the centre texel is
written as `material r g b`: Blender's own evaluation of the node chain,
which `tests/nodes_truth.rs` holds the fold to.
"""
import os

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.join(HERE, "..", "fixtures", "NODES.blend")
OUT = os.path.join(HERE, "nodes.txt")
EMISSIVE = ["MapRanged", "MapClamped", "Separated", "SeparatedHSL", "Combined", "CombinedHSL",
            "Vectored", "Dotted", "Grouped", "GroupedLinked", "Nested"]

bpy.ops.wm.open_mainfile(filepath=FIXTURE, load_ui=False)
scene = bpy.context.scene
scene.render.engine = "CYCLES"
scene.cycles.device = "CPU"
scene.cycles.samples = 1
scene.render.bake.use_pass_direct = False
scene.render.bake.use_pass_indirect = False
scene.render.bake.margin = 0

lines = []
for name in EMISSIVE:
    ob = bpy.data.objects[name]
    mat = ob.data.materials[0]
    img = bpy.data.images.new(name + "_bake", 4, 4, float_buffer=True, is_data=True)
    tex = mat.node_tree.nodes.new("ShaderNodeTexImage")
    tex.image = img
    mat.node_tree.nodes.active = tex
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    bpy.ops.object.bake(type="EMIT")
    px = list(img.pixels)
    i = (2 * 4 + 2) * 4
    lines.append("%s %.6f %.6f %.6f" % (name, px[i], px[i + 1], px[i + 2]))
    print(lines[-1])

with open(OUT, "w") as f:
    f.write("# Cycles %s: Emit bake of each emission plane of NODES.blend (blender_nodes.py)\n" % bpy.app.version_string)
    f.write("\n".join(lines) + "\n")
print("wrote", OUT)
