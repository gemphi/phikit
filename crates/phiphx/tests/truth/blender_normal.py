"""Regenerate tests/truth/normal_map.txt with real Blender (pip install bpy):

    python tests/truth/blender_normal.py -- <files...> | grep '\\.blend|' > tests/truth/normal_map.txt

One line per material whose Principled Normal input comes from a Normal Map
node fed by an Image Texture: file|material|image name|strength.
"""
import os
import sys

import bpy

for p in sys.argv[sys.argv.index("--") + 1:]:
    try:
        bpy.ops.wm.open_mainfile(filepath=p, load_ui=False)
    except RuntimeError as err:
        print(f"# {os.path.basename(p)}: {str(err).splitlines()[0]}", file=sys.stderr)
    for m in sorted(bpy.data.materials, key=lambda m: m.name):
        if m.node_tree is None:
            continue
        bsdf = next((n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED"), None)
        if bsdf is None or not bsdf.inputs["Normal"].is_linked:
            continue
        nm = bsdf.inputs["Normal"].links[0].from_node
        if nm.type != "NORMAL_MAP" or not nm.inputs["Color"].is_linked:
            continue
        tex = nm.inputs["Color"].links[0].from_node
        if tex.type != "TEX_IMAGE" or tex.image is None:
            continue
        print(f"{os.path.basename(p)}|{m.name}|{tex.image.name}|{nm.inputs['Strength'].default_value:.4f}")
