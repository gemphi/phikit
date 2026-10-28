"""Dump what Blender evaluates for objects under the bundled "Smooth by Angle"
Geometry Nodes group, for `tests/smooth_by_angle_truth.rs`:

    blender -b FILE --python blender_smooth_by_angle.py -- OUT.txt [--stride N] [object ...]

With no object names, every mesh object with a NODES modifier is dumped. Per
object: `O name verts faces`, `V` a digest of the evaluated vertex positions
(`sum (i + 1) * (x + 2y + 3z)`, so a stack that lands its vertices elsewhere
is told apart from one that shades them differently), `F` the evaluated
`use_smooth` per face as 0/1, `S` the evaluated `sharp_edge` edges as `a-b`
vertex pairs (a < b), and one `C face vertex nx ny nz` per face corner from
`Mesh.corner_normals` — every `stride`th corner (1: all of them), which keeps
a production mesh's truth small. `tests/fixtures/make_smooth_by_angle.py`
runs the same `dump` on the file it makes, so the fixture and a production
file are checked the same way.
"""
import sys

import bpy


def dump(objects, path, stride=1):
    dg = bpy.context.evaluated_depsgraph_get()
    lines = []
    for ob in objects:
        me = ob.evaluated_get(dg).data
        lines.append(f"O {ob.name} {len(me.vertices)} {len(me.polygons)}")
        digest = sum((i + 1) * (v.co.x + 2.0 * v.co.y + 3.0 * v.co.z) for i, v in enumerate(me.vertices))
        lines.append(f"V {digest:.3f}")
        lines.append("F " + "".join("1" if p.use_smooth else "0" for p in me.polygons))
        sharp = me.attributes.get("sharp_edge")
        pairs = []
        if sharp is not None:
            for e in me.edges:
                if sharp.data[e.index].value:
                    a, b = sorted(e.vertices)
                    pairs.append(f"{a}-{b}")
        lines.append("S " + " ".join(sorted(pairs)))
        normals = me.corner_normals
        for p in me.polygons:
            for li in p.loop_indices:
                if li % stride:
                    continue
                n = normals[li].vector
                lines.append(f"C {p.index} {me.loops[li].vertex_index} {n.x:.6f} {n.y:.6f} {n.z:.6f}")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


if __name__ == "__main__":
    args = sys.argv[sys.argv.index("--") + 1:]
    stride = 1
    if len(args) > 2 and args[1] == "--stride":
        stride, args = int(args[2]), args[:1] + args[3:]
    names = args[1:]
    picked = [
        ob
        for ob in bpy.data.objects
        if ob.type == "MESH"
        and (ob.name in names if names else any(m.type == "NODES" for m in ob.modifiers))
    ]
    dump(picked, args[0], stride)
