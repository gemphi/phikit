"""Blender's own Unwrap ▸ Conformal, for comparison.

A conformal map is only defined up to a similarity, so comparing UVs
number for number is meaningless. What *is* comparable is the distortion:
for every triangle corner, the angle the mesh makes there against the angle
the UVs make. Blender's own worst and mean are recorded here and the Rust
solver has to do at least as well on the same mesh.

Writes `unwrap.txt`:
    mesh <name> <verts> <faces>
    pos <x> <y> <z> ...            (vertex positions, in order)
    face <n> <v0> <v1> ... per face
    blender <worst_deg> <mean_deg>

Run: `python3 -I truth/blender_unwrap.py`
"""
import math
import os
import sys

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "unwrap.txt")


def angle(o, a, b):
    u = [a[k] - o[k] for k in range(len(o))]
    v = [b[k] - o[k] for k in range(len(o))]
    du = math.sqrt(sum(x * x for x in u)) or 1e-12
    dv = math.sqrt(sum(x * x for x in v)) or 1e-12
    c = sum(x * y for x, y in zip(u, v)) / (du * dv)
    return math.acos(max(-1.0, min(1.0, c)))


def distortion(me):
    """Worst and mean |angle3 - angle2| over every triangle corner."""
    uvs = me.uv_layers.active.data
    worst, total, n = 0.0, 0.0, 0
    for poly in me.polygons:
        loops = list(poly.loop_indices)
        p = [me.vertices[me.loops[l].vertex_index].co for l in loops]
        q = [uvs[l].uv for l in loops]
        for k in range(1, len(loops) - 1):
            tri = [0, k, k + 1]
            for j in range(3):
                a, b, c = tri[j], tri[(j + 1) % 3], tri[(j + 2) % 3]
                d = abs(angle(p[a], p[b], p[c]) - angle(q[a], q[b], q[c]))
                worst = max(worst, d)
                total += d
                n += 1
    return math.degrees(worst), math.degrees(total / max(n, 1))


def build_cylinder(rings=6, seg=12):
    """An open cylinder: developable, so a perfect unwrap exists."""
    verts, faces = [], []
    for j in range(rings + 1):
        for i in range(seg + 1):
            t = i / seg * math.tau
            verts.append((math.cos(t), math.sin(t), j / rings * 2.0))
    return verts, quads(rings, seg)


def build_dome(rings=6, seg=12):
    """A spherical cap: NOT developable, so no unwrap is perfect and the
    distortion each solver leaves is a real comparison."""
    verts = []
    for j in range(rings + 1):
        phi = j / rings * (math.pi / 2.5)
        for i in range(seg + 1):
            t = i / seg * math.tau
            verts.append((math.sin(phi) * math.cos(t), math.sin(phi) * math.sin(t), math.cos(phi)))
    return verts, quads(rings, seg)


def build_band(rings=6, seg=12):
    """A band of a sphere, between two latitudes: not developable either,
    and with no degenerate pole row, so every triangle has real area."""
    verts = []
    for j in range(rings + 1):
        phi = 0.35 + j / rings * 0.85
        for i in range(seg + 1):
            t = i / seg * math.tau
            verts.append((math.sin(phi) * math.cos(t), math.sin(phi) * math.sin(t), math.cos(phi)))
    return verts, quads(rings, seg)


def quads(rings, seg):
    stride = seg + 1
    return [(j * stride + i, j * stride + i + 1, j * stride + i + stride + 1, j * stride + i + stride)
            for j in range(rings) for i in range(seg)]


def one(name, verts, faces, fh):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    me = bpy.data.meshes.new(name)
    me.from_pydata([list(v) for v in verts], [], [list(f) for f in faces])
    me.update()
    ob = bpy.data.objects.new(name, me)
    bpy.context.collection.objects.link(ob)
    bpy.context.view_layer.objects.active = ob
    ob.select_set(True)

    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    try:
        bpy.ops.uv.unwrap(method="CONFORMAL")
    except TypeError:
        # 4.x+ spells it differently in places; ANGLE_BASED ends in LSCM too.
        bpy.ops.uv.unwrap(method="ANGLE_BASED")
    bpy.ops.object.mode_set(mode="OBJECT")

    worst, mean = distortion(me)
    fh.write(f"mesh {name} {len(verts)} {len(faces)}\n")
    for v in verts:
        fh.write("pos " + " ".join(f"{c:.7g}" for c in v) + "\n")
    for f in faces:
        fh.write("face " + " ".join(str(k) for k in f) + "\n")
    fh.write(f"blender {worst:.5f} {mean:.5f}\n")
    print(f"{name}: worst {worst:.3f} deg, mean {mean:.3f} deg", flush=True)


def main():
    with open(OUT, "w", encoding="utf-8") as fh:
        v, f = build_cylinder()
        one("Tube", v, f, fh)
        v, f = build_dome()
        one("Dome", v, f, fh)
        v, f = build_band()
        one("Band", v, f, fh)
    print("wrote", OUT)


if __name__ == "__main__":
    sys.exit(main())
