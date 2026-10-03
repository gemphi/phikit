"""Write the Output-properties fixture, and dump what Blender reads back.

Run with Blender 5.0.1:  python3 blender_output.py

Produces `tests/fixtures/OUTPUT-FORMAT.blend` — a scene whose Output panel is
set to none of its defaults — and `tests/truth/output_format.txt`, the values
Blender itself reports for it and for the other fixtures. `output.rs` must
read the same numbers off the file without Blender.
"""

import os
import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.normpath(os.path.join(HERE, "..", "fixtures"))


def make():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    r = bpy.context.scene.render
    r.filepath = "//shot/frame"
    imf = r.image_settings
    imf.file_format = "OPEN_EXR"
    imf.color_depth = "32"
    imf.color_mode = "RGB"
    imf.exr_codec = "PIZ"
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(FIXTURE, "OUTPUT-FORMAT.blend"))


def report(path):
    bpy.ops.wm.open_mainfile(filepath=path)
    r = bpy.context.scene.render
    imf = r.image_settings
    return "|".join(
        [
            os.path.basename(path),
            r.filepath,
            imf.file_format,
            imf.color_depth,
            imf.color_mode,
            imf.exr_codec,
            str(imf.quality),
            str(imf.compression),
        ]
    )


if __name__ == "__main__":
    make()
    rows = []
    for name in sorted(os.listdir(FIXTURE)):
        if name.endswith(".blend"):
            rows.append(report(os.path.join(FIXTURE, name)))
    out = os.path.join(HERE, "output_format.txt")
    with open(out, "w") as fh:
        fh.write("\n".join(rows) + "\n")
    print("\n".join(rows))
