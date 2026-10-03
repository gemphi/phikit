"""Regenerate tests/truth/base_image.txt with real Blender (pip install bpy):

    python tests/truth/blender_base_image.py -- <files...> | grep '\\.blend|' > tests/truth/base_image.txt

For every material with a Principled BSDF: the first Image Texture found
walking upstream from `Base Color` (breadth-first, sockets in order, not
through Bump / Normal Map nodes). Key = image filepath, else its name; empty
when there is none. Line: file|material|key.
"""
import os
import sys

import bpy

SKIP = {"BUMP", "NORMAL_MAP"}


def base_image(bsdf):
    queue, seen = [bsdf.inputs["Base Color"]], set()
    while queue:
        sock = queue.pop(0)
        if not sock.is_linked:
            continue
        node = sock.links[0].from_node
        if node.name in seen or node.type in SKIP:
            continue
        seen.add(node.name)
        if node.type == "TEX_IMAGE":
            if node.image is not None:
                return node.image.filepath or node.image.name
            continue
        queue.extend(node.inputs)
    return ""


for p in sys.argv[sys.argv.index("--") + 1:]:
    try:
        bpy.ops.wm.open_mainfile(filepath=p, load_ui=False)
    except RuntimeError as err:
        print(f"# {os.path.basename(p)}: {str(err).splitlines()[0]}", file=sys.stderr)
    for m in sorted(bpy.data.materials, key=lambda m: m.name):
        if m.node_tree is None:
            continue
        bsdf = next((n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED"), None)
        if bsdf is not None:
            print(f"{os.path.basename(p)}|{m.name}|{base_image(bsdf)}")
