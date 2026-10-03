"""Regenerate tests/truth/shade_maps.txt with real Blender (pip install bpy):

    python tests/truth/blender_maps.py -- <files...> | grep '\\.blend|' > tests/truth/shade_maps.txt

One line per material and mapped Principled input fed by an Image Texture
(directly or through intermediate nodes): file|material|socket|image name.
"""
import os
import sys

import bpy

SOCKETS = ["Roughness", "Metallic", "Emission Color"]


def upstream_image(socket):
    """First Image Texture upstream of `socket`, breadth-first."""
    queue = [socket]
    seen = set()
    while queue:
        s = queue.pop(0)
        if not s.is_linked:
            continue
        node = s.links[0].from_node
        if node.name in seen or node.type in {"BUMP", "NORMAL_MAP"}:
            continue
        seen.add(node.name)
        if node.type == "TEX_IMAGE":
            return node.image
        queue.extend(node.inputs)
    return None


for p in sys.argv[sys.argv.index("--") + 1:]:
    try:
        bpy.ops.wm.open_mainfile(filepath=p, load_ui=False)
    except RuntimeError as err:
        print(f"# {os.path.basename(p)}: {str(err).splitlines()[0]}", file=sys.stderr)
    for m in sorted(bpy.data.materials, key=lambda m: m.name):
        if m.node_tree is None:
            continue
        bsdf = next((n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED"), None)
        if bsdf is None:
            continue
        for name in SOCKETS:
            im = upstream_image(bsdf.inputs[name])
            if im is not None:
                print(f"{os.path.basename(p)}|{m.name}|{name}|{im.name}")
