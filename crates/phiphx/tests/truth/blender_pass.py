"""Render a multi-layer EXR with every pass on, and dump its channel names.

Run with Blender 5.0.1:  python3 blender_pass.py

The point is not the picture — it is what Blender *calls* each pass inside an
EXR. Those names are what a compositor looks for, and several of them are not
guessable: Mist's single channel is "Z" rather than "V", the two index passes
use "X", and Ambient Occlusion is three channels, not one.
"""

import math
import os
import re

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = "/tmp/phiber_pass_truth"


def render():
    os.makedirs(OUT, exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 1
    scene.render.resolution_x = 16
    scene.render.resolution_y = 16
    scene.render.image_settings.media_type = "MULTI_LAYER_IMAGE"
    scene.render.image_settings.file_format = "OPEN_EXR_MULTILAYER"
    scene.render.filepath = os.path.join(OUT, "shot")

    layer = scene.view_layers[0]
    for name in dir(layer):
        if name.startswith("use_pass_"):
            try:
                setattr(layer, name, True)
            except (AttributeError, TypeError):
                pass

    bpy.ops.mesh.primitive_cube_add()
    data = bpy.data.cameras.new("Camera")
    cam = bpy.data.objects.new("Camera", data)
    scene.collection.objects.link(cam)
    cam.location = (0.0, -6.0, 0.0)
    cam.rotation_euler = (math.radians(90.0), 0.0, 0.0)
    scene.camera = cam

    bpy.ops.render.render(write_still=True)
    return os.path.join(OUT, "shot.exr")


def channels(path):
    """Channel names out of the EXR header, without an EXR library."""
    raw = open(path, "rb").read()
    found = re.findall(rb"ViewLayer\.[A-Za-z][A-Za-z0-9 ]*\.[A-Za-z]", raw)
    return sorted({name.decode() for name in found})


if __name__ == "__main__":
    rows = channels(render())
    with open(os.path.join(HERE, "pass_channels.txt"), "w") as fh:
        fh.write("\n".join(rows) + "\n")
    print("\n".join(rows))
