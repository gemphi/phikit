"""Regenerate tests/truth/mesh_digest.txt with real Blender (pip install bpy):

    python tests/truth/blender_mesh_digest.py -- <files...> > tests/truth/mesh_digest.txt

One line per mesh object in the scene the file opens with: file|object|verts|faces|positions|faces|uvs|material|smooth.
The digest formulas must match tests/blender_truth.rs.
"""
import os
import sys

import bpy

for p in sys.argv[sys.argv.index("--") + 1:]:
    try:
        bpy.ops.wm.open_mainfile(filepath=p, load_ui=False)
    except RuntimeError as err:  # load reports (e.g. dropped shape keys) still load the file
        print(f"# {os.path.basename(p)}: {str(err).splitlines()[0]}", file=sys.stderr)
    for o in bpy.context.scene.objects:  # the scene the file opens with
        if o.type != "MESH":
            continue
        m = o.data
        ps = sum((i + 1) * (v.co.x + 2 * v.co.y + 3 * v.co.z) for i, v in enumerate(m.vertices))
        fs = sum((i + 1) * sum((k + 1) * x for k, x in enumerate(f.vertices)) for i, f in enumerate(m.polygons))
        uvl = m.uv_layers.active
        uv = sum(((i % 7) + 1) * (d.uv[0] + 2 * d.uv[1]) for i, d in enumerate(uvl.data)) if uvl else 0.0
        mi = sum((i + 1) * f.material_index for i, f in enumerate(m.polygons))
        sm = sum(1 for f in m.polygons if f.use_smooth)
        line = f"{os.path.basename(p)}|{o.name}|{len(m.vertices)}|{len(m.polygons)}|{ps:.1f}|{fs}|{uv:.1f}|{mi}|{sm}"
        print(line.replace("|-0.0|", "|0.0|"))
