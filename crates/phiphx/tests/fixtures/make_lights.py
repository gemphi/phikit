"""LIGHTS.blend: one light of each kind, each with a shape the default
would not give, so `tests/light_truth.rs` can tell a read value from a
fallback (`blend/object/data.rs`, `scene/import_extras.rs`).

  Sun    — Angle 10° (Blender's default is 0.526°), energy 3
  Bulb    — Point, Radius 0.4 (default 0.1), energy 200
  Panel  — Area, Square, Size 2.5 (default 0.25), energy 500
  Cone   — Spot, Radius 0.2, Spot Size 60° (default 45°), Blend 0.6
            (default 0.15), energy 300
  Sharp  — Spot, Blend 0 — a hard-edged cone
  Strip  — Area, Rectangle, Size 3 × 0.5, the object scaled 2 in X
  Round  — Area, Disk, Size 1.5 (its Size Y is ignored, left at 0.25)
  Oval   — Area, Ellipse, Size 2 × 0.75
  Squashed — Spot, Spot Size 50°, the object scaled 1.5 in Y: EEVEE's cone
            follows the object's scale, so this one lights an ellipse

Run with bpy available (pip install bpy):
  python make_lights.py
"""

import math

import bpy

bpy.ops.wm.read_factory_settings(use_empty=True)


def lamp(name, kind, energy, at, **fields):
    data = bpy.data.lights.new(name, type=kind)
    data.energy = energy
    for k, v in fields.items():
        setattr(data, k, v)
    ob = bpy.data.objects.new(name, data)
    ob.location = at
    bpy.context.collection.objects.link(ob)
    return ob


lamp("Sun", "SUN", 3.0, (0.0, 0.0, 5.0), angle=math.radians(10.0))
lamp("Bulb", "POINT", 200.0, (2.0, 0.0, 3.0), shadow_soft_size=0.4)
lamp("Panel", "AREA", 500.0, (-2.0, 0.0, 3.0), size=2.5)
lamp("Cone", "SPOT", 300.0, (0.0, 2.0, 3.0), shadow_soft_size=0.2, spot_size=math.radians(60.0), spot_blend=0.6)
lamp("Sharp", "SPOT", 300.0, (0.0, -2.0, 3.0), spot_blend=0.0)
# Blender reads Size Y only for Rectangle and Ellipse, so Round keeps the
# default 0.25 and must still read back as 1.5 both ways.
strip = lamp("Strip", "AREA", 100.0, (4.0, 0.0, 3.0), shape="RECTANGLE", size=3.0, size_y=0.5)
strip.scale = (2.0, 1.0, 1.0)
lamp("Round", "AREA", 100.0, (-4.0, 0.0, 3.0), shape="DISK", size=1.5)
lamp("Oval", "AREA", 100.0, (0.0, -4.0, 3.0), shape="ELLIPSE", size=2.0, size_y=0.75)
# A spot's cone is round only while its object's scale is uniform
# (`eevee_light.cc`: `spot_size_inv = scale.z / scale.xy`).
squashed = lamp("Squashed", "SPOT", 300.0, (0.0, 4.0, 3.0), spot_size=math.radians(50.0))
squashed.scale = (1.0, 1.5, 1.0)

bpy.ops.wm.save_as_mainfile(filepath=__file__.replace("make_lights.py", "LIGHTS.blend"), compress=True)
