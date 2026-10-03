"""Write the depth-of-field fixture and dump what Blender reads back.

Run with Blender 5.0.1:  python3 blender_dof.py

`DEPTH-OF-FIELD.blend` has a camera focusing on an Empty and a second one
focusing by distance, neither left at a default, so a field read at the wrong
offset in `CameraDOFSettings` cannot pass by accident.
"""

import math
import os
import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.normpath(os.path.join(HERE, "..", "fixtures"))
BLEND = os.path.join(FIXTURE, "DEPTH-OF-FIELD.blend")


def make():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    target = bpy.data.objects.new("FocusTarget", None)
    bpy.context.scene.collection.objects.link(target)
    # Deliberately off the camera axis: the focus distance is the projection
    # onto that axis (10), not the straight-line distance (about 10.77), and
    # a fixture on the axis could not tell the two apart.
    target.location = (4.0, 7.0, 0.0)

    # Focuses on the Empty, hexagonal iris, anamorphic.
    data = bpy.data.cameras.new("TrackingCam")
    data.lens = 85.0
    data.dof.use_dof = True
    data.dof.focus_object = target
    data.dof.aperture_fstop = 1.8
    data.dof.aperture_blades = 6
    data.dof.aperture_rotation = math.radians(15.0)
    data.dof.aperture_ratio = 1.6
    obj = bpy.data.objects.new("TrackingCam", data)
    bpy.context.scene.collection.objects.link(obj)
    obj.location = (0.0, -3.0, 0.0)
    obj.rotation_euler = (math.radians(90.0), 0.0, 0.0)

    # Focuses by distance, round iris.
    data = bpy.data.cameras.new("DistanceCam")
    data.lens = 35.0
    data.dof.use_dof = True
    data.dof.focus_distance = 4.25
    data.dof.aperture_fstop = 5.6
    obj = bpy.data.objects.new("DistanceCam", data)
    bpy.context.scene.collection.objects.link(obj)
    obj.location = (3.0, -3.0, 0.0)

    # A third with depth of field switched off, so "off" is covered too.
    data = bpy.data.cameras.new("PinholeCam")
    data.dof.use_dof = False
    data.dof.aperture_fstop = 1.2
    obj = bpy.data.objects.new("PinholeCam", data)
    bpy.context.scene.collection.objects.link(obj)

    bpy.ops.wm.save_as_mainfile(filepath=BLEND)


def report():
    bpy.ops.wm.open_mainfile(filepath=BLEND)
    rows = []
    for obj in sorted(bpy.data.objects, key=lambda o: o.name):
        if obj.type != "CAMERA":
            continue
        dof = obj.data.dof
        rows.append(
            "|".join(
                [
                    obj.name,
                    "on" if dof.use_dof else "off",
                    "%.4f" % dof.focus_distance,
                    dof.focus_object.name if dof.focus_object else "",
                    "%.4f" % dof.aperture_fstop,
                    str(dof.aperture_blades),
                    "%.4f" % dof.aperture_rotation,
                    "%.4f" % dof.aperture_ratio,
                    "%.4f" % obj.data.lens,
                ]
            )
        )
    return rows


if __name__ == "__main__":
    make()
    rows = report()
    with open(os.path.join(HERE, "depth_of_field.txt"), "w") as fh:
        fh.write("\n".join(rows) + "\n")
    print("\n".join(rows))
