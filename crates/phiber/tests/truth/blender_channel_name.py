"""Every animatable property's UI name and array letters, from Blender itself.

A channel row's label is not stored in a `.blend`: `getname_anim_fcurve`
(`editors/animation/anim_ipo_utils.cc`) BUILDS it, every draw, out of three
pieces -- the array item's letter, `RNA_property_ui_name(prop)`, and the
struct that owns the property. The middle piece is a *declared* string:
`energy` displays as "Power", `spot_size` as "Beam Angle", `mid_level` as
"Midlevel". Nothing derives those from the identifier, so the only honest
source is Blender.

This writes `channel_name.txt`:

    owner <TAB> identifier <TAB> ui name <TAB> letters <TAB> array length

`letters` is `RNA_property_array_item_char` for the property's subtype
(`makesrna/intern/rna_access.cc:1370`), which is a *four* letter string even
for a three component array -- the function only checks `index < 4`:

    QUATERNION, AXISANGLE                      -> WXYZ
    TRANSLATION, DIRECTION, XYZ, XYZ_LENGTH,
    EULER, VELOCITY, ACCELERATION, COORDS,
    PIXEL                                      -> XYZW
    COLOR, COLOR_GAMMA                         -> RGBA
    anything else                              -> (none)

An empty `letters` with an array length above 1 is a real case: such a
property takes Blender's bracketed `[%d]` form at every index.

The owners are the ones a phiber channel list can reach -- the object and
pose bone that carry transforms, the datablocks phiber evaluates drivers on
(`phiphx/src/blend/{object/light_driver,shade/driver,shade/world_driver}.rs`),
the modifier types in `phiphx/src/blend/modifier/driver.rs`'s `SETTINGS`, and
every constraint type plus the shape key, because `influence`, `mute` and
`value` are the three most commonly driven properties in any rig and none of
them belong to the object.

Run (Blender 5.x, or the `bpy` module at the same version):

    blender --background --factory-startup --python blender_channel_name.py \
        > channel_name.txt
"""

import collections
import sys

import bpy

# RNA_property_array_item_char, exactly: the index bound is 4 regardless of
# how many components the property actually has.
LETTERS = {
    "QUATERNION": "WXYZ",
    "AXISANGLE": "WXYZ",
    "TRANSLATION": "XYZW",
    "DIRECTION": "XYZW",
    "XYZ": "XYZW",
    "XYZ_LENGTH": "XYZW",
    "EULER": "XYZW",
    "VELOCITY": "XYZW",
    "ACCELERATION": "XYZW",
    "COORDS": "XYZW",
    "PIXEL": "XYZW",
    "COLOR": "RGBA",
    "COLOR_GAMMA": "RGBA",
}

# The modifier types `phiphx/src/blend/modifier/driver.rs` evaluates. Blender
# has sixty-odd; a name for one phiber cannot evaluate would be a row no
# channel list ever shows.
MODIFIER = ("SUBSURF", "DISPLACE", "SOLIDIFY", "WELD", "MIRROR", "ARRAY", "TRIANGULATE", "LATTICE")

rows = collections.defaultdict(dict)  # identifier -> {owner: (name, letters, length)}


def dump(owner, label):
    """Every animatable property `owner` carries, as RNA declares it.

    `owner` is an instance, or a `bpy.types` class for a struct that cannot
    be instantiated standalone -- `bl_rna` answers for both.
    """
    for prop in owner.bl_rna.properties:
        # A channel drives a number or a flag; a string or a pointer cannot
        # hold an F-curve, and a read-only property cannot be driven.
        if prop.is_readonly or prop.type not in ("FLOAT", "INT", "BOOLEAN"):
            continue
        length = getattr(prop, "array_length", 0)
        letters = LETTERS.get(getattr(prop, "subtype", "NONE"), "") if length > 1 else ""
        # `prop.name == prop.identifier` is RNA's own fallback showing
        # through, which is what the Rust side falls back to as well -- a
        # row for it would say nothing.
        if prop.name == prop.identifier:
            continue
        rows[prop.identifier][label] = (prop.name, letters, length)


bpy.ops.wm.read_factory_settings(use_empty=True)

bpy.ops.mesh.primitive_cube_add()
cube = bpy.context.object
dump(cube, "Object")

bpy.ops.object.armature_add()
dump(bpy.context.object.pose.bones[0], "PoseBone")

for kind in ("POINT", "SUN", "SPOT", "AREA"):
    bpy.ops.object.light_add(type=kind)
    dump(bpy.context.object.data, "Light")

dump(bpy.data.materials.new("M"), "Material")
dump(bpy.data.worlds.new("W"), "World")

for kind in MODIFIER:
    dump(cube.modifiers.new(name=kind, type=kind), f"Mod:{kind}")

# Constraints: `influence` and `mute` live here, not on the object, and a
# driven constraint setting is how most rigs are wired.
for kind in sorted(
    item.identifier for item in bpy.types.Constraint.bl_rna.properties["type"].enum_items
):
    try:
        dump(cube.constraints.new(type=kind), f"Con:{kind}")
    except (RuntimeError, TypeError) as why:  # a type an object cannot take
        print(f"# constraint {kind}: {why}", file=sys.stderr)

# `ConstraintTargetBone.weight` -- an Armature constraint's `targets[...]`,
# whose "Blend Weight" is not the "Weight" an IK constraint carries.
dump(bpy.types.ConstraintTargetBone, "ConstraintTargetBone")

# `ShapeKey.value` -- a driven shape key is the other half of a rig.
bpy.context.view_layer.objects.active = cube
cube.shape_key_add(name="Basis")
dump(cube.shape_key_add(name="Key"), "ShapeKey")

agree, conflict = {}, {}
for identifier, by_owner in rows.items():
    value = set(by_owner.values())
    (agree if len(value) == 1 else conflict)[identifier] = (
        next(iter(value)) if len(value) == 1 else by_owner
    )

print(
    f"# {len(rows)} identifiers: {len(agree)} agree across owners, {len(conflict)} conflict",
    file=sys.stderr,
)
for identifier, by_owner in sorted(conflict.items()):
    spelling = " | ".join(f"{owner}={value[0]!r}" for owner, value in sorted(by_owner.items()))
    print(f"#   {identifier}: {spelling}", file=sys.stderr)

for identifier in sorted(rows):
    for owner, (name, letters, length) in sorted(rows[identifier].items()):
        print(f"{owner}\t{identifier}\t{name}\t{letters}\t{length}")
