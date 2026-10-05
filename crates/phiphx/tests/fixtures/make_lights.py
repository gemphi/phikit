"""LIGHTS.blend: one light of each kind, each with a shape the default
would not give, so `tests/light_truth.rs` can tell a read value from a
fallback (`blend/object/data.rs`, `scene/import_extras.rs`).

  Sun    — Angle 10° (Blender's default is 0.526°), energy 3
  Bulb    — Point, Radius 0.4 (default 0.1), energy 200
  Panel  — Area, Size 2.5 (default 0.25), energy 500
  Cone   — Spot, Radius 0.2, Spot Size 60° (default 45°), energy 300

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
lamp("Cone", "SPOT", 300.0, (0.0, 2.0, 3.0), shadow_soft_size=0.2, spot_size=math.radians(60.0))

bpy.ops.wm.save_as_mainfile(filepath=__file__.replace("make_lights.py", "LIGHTS.blend"), compress=True)
