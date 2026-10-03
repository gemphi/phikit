"""Blender's evaluated world-space vertices for named objects, as the
reference `phiber --example vertex_delta` compares against.

    python3 blender_vertex_dump.py FILE OUT.bin NAME [NAME...] [--keep N]

`--keep N` mutes every modifier after the first N, so a stack can be compared
one stage at a time — which is how a deviation is pinned to one modifier.

Format: for each object in the order given, `u32` vertex count followed by
that many `f32[3]` world-space positions, little-endian.
"""

import struct
import sys

import bpy
from mathutils import Vector

argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else sys.argv[1:]
keep = None
if "--keep" in argv:
    i = argv.index("--keep")
    keep = int(argv[i + 1])
    argv = argv[:i] + argv[i + 2 :]
path, out, names = argv[0], argv[1], argv[2:]

bpy.ops.wm.open_mainfile(filepath=path)

if keep is not None:
    for name in names:
        for i, m in enumerate(bpy.data.objects[name].modifiers):
            m.show_viewport = i < keep

deps = bpy.context.evaluated_depsgraph_get()
with open(out, "wb") as f:
    for name in names:
        ob = bpy.data.objects[name]
        evaluated = ob.evaluated_get(deps)
        me = evaluated.to_mesh()
        count = len(me.vertices)
        f.write(struct.pack("<I", count))
        for v in me.vertices:
            p = ob.matrix_world @ Vector(v.co)
            f.write(struct.pack("<3f", p.x, p.y, p.z))
        evaluated.to_mesh_clear()
        print(f"dumped {name!r} {count} vertices")
