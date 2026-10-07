# Builds `fixtures/MODDRIVERS.blend`: drivers on modifier settings across the
# modifier types phiphx reads, covering both value kinds and every clamp.
#
#   python -I blender_mod_driver_fixture.py <out.blend>
#
# A modifier driver lives on the **object's** `AnimData`, pathed
# `modifiers["<name>"].<property>` — not on the modifier and not on the mesh.
# So one object can carry drivers for its whole stack, and the modifier's
# *type* is not in the path: it has to be resolved from the stack by name,
# because the same property name means different things on different
# modifiers (`strength` on Displace and on Lattice, `merge_threshold` on
# Weld and on Mirror, `offset` on Solidify and on Array).
#
# Two value kinds, and the integer rule is the one worth measuring:
# a driver's value is a float, and an integer property takes it by
# **truncation toward zero** — not rounding, and not floor. 0.7 → 0,
# 2.5 → 2, −0.7 → 0, −1.5 → −1. Then the hard range clamps.
import bpy, sys

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
out = args[0] if args else "MODDRIVERS.blend"

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene
sc.frame_start, sc.frame_end = 1, 21

# ── SOURCE: keyed on fractional values with CONSTANT interpolation, so each
# sampled frame lands on an exact number and the truncation is unambiguous.
bpy.ops.mesh.primitive_cube_add(size=1.0)
src = bpy.context.object
src.name = "SOURCE"
STEPS = {1: -0.7, 6: 2.5, 11: 0.5, 16: 11.9, 21: -2.5}
for frame, z in STEPS.items():
    src.location = (0.0, 0.0, z)
    src.keyframe_insert("location", frame=frame)
for curve in [c for lay in src.animation_data.action.layers for st in lay.strips for cb in st.channelbags for c in cb.fcurves]:
    for key in curve.keyframe_points:
        key.interpolation = "CONSTANT"


def driven(target, prop, index=-1, expression="z"):
    fcurve = target.driver_add(prop) if index < 0 else target.driver_add(prop, index)
    fcurve.driver.type = "SCRIPTED"
    fcurve.driver.expression = expression
    var = fcurve.driver.variables.new()
    var.name = "z"
    var.type = "TRANSFORMS"
    t = var.targets[0]
    t.id = src
    t.transform_type = "LOC_Z"
    t.transform_space = "WORLD_SPACE"
    return fcurve


# A plane, so a driven Subsurf level cannot explode the geometry.
bpy.ops.mesh.primitive_plane_add(size=2.0, location=(3, 0, 0))
ob = bpy.context.object
ob.name = "OB_MODS"

# ── Integer settings: the truncation and the three different hard minima.
subsurf = ob.modifiers.new(name="Subsurf", type="SUBSURF")
driven(subsurf, "levels")            # INT 0 .. 11
driven(subsurf, "quality")           # INT 1 .. 10
driven(subsurf, "render_levels")     # INT 0 .. 11
triangulate = ob.modifiers.new(name="Tri", type="TRIANGULATE")
driven(triangulate, "min_vertices")  # INT 4 .. ∞ — a non-zero minimum

# ── Float settings, unbounded both ways.
displace = ob.modifiers.new(name="Disp", type="DISPLACE")
driven(displace, "strength")         # FLOAT unbounded
driven(displace, "mid_level")
solidify = ob.modifiers.new(name="Sol", type="SOLIDIFY")
driven(solidify, "thickness")        # FLOAT unbounded
driven(solidify, "offset")
# `material_offset` is a SIGNED integer, and the only property here whose
# range reaches below zero — which is what tells truncation from floor.
driven(solidify, "material_offset")  # INT −32768 .. 32767

# ── Floats that clamp at zero.
weld = ob.modifiers.new(name="Weld", type="WELD")
driven(weld, "merge_threshold")      # FLOAT 0 .. ∞
mirror = ob.modifiers.new(name="Mirror", type="MIRROR")
driven(mirror, "merge_threshold")    # FLOAT 0 .. ∞ — same name, other modifier
driven(mirror, "bisect_threshold")

# ── A vector setting, by component.
array = ob.modifiers.new(name="Array", type="ARRAY")
driven(array, "constant_offset_displace", 0)
driven(array, "constant_offset_displace", 2)
driven(array, "relative_offset_displace", 1)

# ── A muted driver: the saved value stands.
lattice_ob_mesh = bpy.data.lattices.new("Lat")
lattice_ob = bpy.data.objects.new("OB_LAT", lattice_ob_mesh)
sc.collection.objects.link(lattice_ob)
lattice = ob.modifiers.new(name="Lat", type="LATTICE")
lattice.object = lattice_ob
lattice.strength = 0.625
muted = driven(lattice, "strength")
muted.mute = True

# ── A second object, so the per-object split is covered rather than assumed.
bpy.ops.mesh.primitive_plane_add(size=2.0, location=(6, 0, 0))
other = bpy.context.object
other.name = "OB_OTHER"
other_disp = other.modifiers.new(name="Disp", type="DISPLACE")
driven(other_disp, "strength", expression="z * 0.5")

bpy.ops.wm.save_as_mainfile(filepath=out, compress=True)
print("wrote", out)
