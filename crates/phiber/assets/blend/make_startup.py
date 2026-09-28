"""Write Blender's factory startup scene as a compressed .blend, with the
Blender this runs in — the startup file phiber's New File reads for that
major version (`scene/template.rs`).

    python -m venv v && ./v/bin/pip install bpy==4.2.23 && ./v/bin/python make_startup.py startup-4.blend
    pip install bpy==5.0.1 && python make_startup.py startup-5.blend

PyPI carries `bpy` 4.2 and up only. 3.6's template is the factory file in
Blender's own repository — `git show v3.6.23:release/datafiles/startup.blend`
— zstd-wrapped as a compressed save (`zstandard.ZstdCompressor(3)`).
"""

import os
import sys

import bpy

out = sys.argv[-1]
bpy.ops.wm.read_factory_settings()
names = sorted(o.name for o in bpy.data.objects)
scene = bpy.context.scene
print("Blender", bpy.app.version_string, "objects", names, "frames", scene.frame_start, scene.frame_end)
assert names == ["Camera", "Cube", "Light"], names
bpy.ops.wm.save_as_mainfile(filepath=os.path.abspath(out), compress=True, copy=True)
print("wrote", out, os.path.getsize(out), "bytes")
