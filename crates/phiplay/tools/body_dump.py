"""Dumps the players' figure — `phiber/assets/source/epic01-player.blend`'s
body, its vertex groups and weights, and its skeleton's bones — for the
`figure` example. python3 tools/body_dump.py <player.blend> <out.bin>"""
import bpy, struct, sys

bpy.ops.wm.open_mainfile(filepath=sys.argv[-2])
body = next(o for o in bpy.data.objects if o.type == "MESH" and o.name.endswith("Body"))
rig = next(o for o in bpy.data.objects if o.type == "ARMATURE")
me = body.data
out = bytearray()
u32 = lambda v: out.extend(struct.pack("<I", v))
f32 = lambda *v: out.extend(struct.pack(f"<{len(v)}f", *v))
text = lambda s: (u32(len(s.encode())), out.extend(s.encode()))
u32(len(me.vertices))
for v in me.vertices:
    f32(*(body.matrix_world @ v.co))
u32(len(me.polygons))
for p in me.polygons:
    u32(len(p.vertices))
    for i in p.vertices:
        u32(i)
u32(len(body.vertex_groups))
for g in body.vertex_groups:
    text(g.name)
for v in me.vertices:
    u32(len(v.groups))
    for g in v.groups:
        u32(g.group); f32(g.weight)
bones = [b for b in rig.data.bones]
u32(len(bones))
for b in bones:
    text(b.name)
    f32(*(rig.matrix_world @ b.head_local)); f32(*(rig.matrix_world @ b.tail_local))
open(sys.argv[-1], "wb").write(out)
print("dumped", len(me.vertices), "vertices,", len(bones), "bones")
