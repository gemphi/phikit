"""Surface Deform bound to a target that is then changed, for
`tests/surface_deform_truth.rs`.

Target "Skin": a 3 m disk, a 12-gon inset twice into two rings of quads
around an inner 12-gon, four quads cut into triangles, so the bind holds
every mode — corner triangles, n-gons and centroids — at (0.5, 0.2, 0.1),
rotated, scaled 1.3.

Bound meshes, each a 1.8 m grid of 9 x 9 vertices at its own transform
above the sheet, bound with `surfacedeform_bind`, then the target's mesh
is changed (a sine ripple over every vertex) so each must move:
- "Wrap": strength 1.
- "Wrap Half": strength 0.6.
- "Wrap Group": a vertex group ramping 0..1 along X, inverted.
- "Wrap Twist": bound to "Twisted", a copy of the sheet under a Simple
  Deform twist, whose angle is then doubled — the target's EVALUATED mesh.
- "Unbound": the modifier never bound (the grid stays flat).
- "Lost": bound, then its target cleared (Blender disables it; the grid
  stays flat).

Blender's evaluated vertices are dumped by `../truth/blender_eval_dump.py`
into `surface_deform_truth.txt` from a second Blender run on the saved
file.

Run: blender -b --python make_surface_deform.py -- surface_deform.blend"""
import bpy, bmesh, sys, os, math

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene


def link(name, mesh):
    ob = bpy.data.objects.new(name, mesh)
    sc.collection.objects.link(ob)
    return ob


def sheet(name):
    """A 3 m disk: a 12-gon inset twice (a 12-gon in two rings of quads,
    n-gon binds) with four of the outer quads cut into triangles
    (corner-triangle binds), so the bind holds every mode. No two adjacent
    edges are collinear, which the bind refuses."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_circle(bm, cap_ends=True, segments=12, radius=1.5)
    for thickness in (0.45, 0.4):
        bm.faces.ensure_lookup_table()
        big = max(bm.faces, key=lambda f: len(f.verts))
        bmesh.ops.inset_individual(bm, faces=[big], thickness=thickness)
    bm.faces.ensure_lookup_table()
    quads = [f for f in bm.faces if len(f.verts) == 4]
    bmesh.ops.triangulate(bm, faces=quads[:4])
    bm.to_mesh(me)
    bm.free()
    ob = link(name, me)
    ob.location = (0.5, 0.2, 0.1)
    ob.rotation_euler = (0.2, 0.1, 0.4)
    ob.scale = (1.3, 1.3, 1.3)
    return ob


def grid(name, loc, group=False):
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_grid(bm, x_segments=8, y_segments=8, size=0.9)
    bm.to_mesh(me)
    bm.free()
    ob = link(name, me)
    ob.location = loc
    ob.rotation_euler = (0.25, 0.05, 0.4)
    if group:
        vg = ob.vertex_groups.new(name="Ramp")
        for v in me.vertices:
            vg.add([v.index], (v.co.x + 1.5) / 3.0, "REPLACE")
    return ob


def bind(ob, target, strength=1.0, group=None, invert=False, do_bind=True):
    md = ob.modifiers.new("Surface", "SURFACE_DEFORM")
    md.target = target
    md.strength = strength
    if group:
        md.vertex_group = group
        md.invert_vertex_group = invert
    if do_bind:
        # The bind runs on the EVALUATED object, so the depsgraph must know
        # the modifier first.
        bpy.context.view_layer.update()
        with bpy.context.temp_override(object=ob, active_object=ob, selected_objects=[ob]):
            bpy.ops.object.surfacedeform_bind(modifier=md.name)
        bpy.context.view_layer.update()
        assert md.is_bound, ob.name
    return md


skin = sheet("Skin")
twisted = sheet("Twisted")
twisted.location = (6.0, 0.0, 0.0)
tw = twisted.modifiers.new("Twist", "SIMPLE_DEFORM")
tw.deform_method, tw.angle, tw.deform_axis = "TWIST", math.radians(30), "Z"

bind(grid("Wrap", (0.4, 0.1, 0.8)), skin)
bind(grid("Wrap Half", (0.6, 0.3, 0.7)), skin, strength=0.6)
bind(grid("Wrap Group", (0.3, -0.1, 0.9), group=True), skin, group="Ramp", invert=True)
bind(grid("Wrap Twist", (6.2, 0.1, 0.8)), twisted)
bind(grid("Unbound", (0.5, 0.0, 0.6)), skin, do_bind=False)
lost = bind(grid("Lost", (0.5, 0.2, 1.0)), skin)

# Now change the targets: a ripple over the sheet's vertices, and a
# stronger twist on the copy.
for me in (skin.data, twisted.data):
    for v in me.vertices:
        v.co.z += 0.15 * math.sin(3.0 * v.co.x) * math.cos(2.0 * v.co.y)
        v.co.x += 0.05 * math.cos(4.0 * v.co.z)
tw.angle = math.radians(60)
lost.target = None

path = sys.argv[sys.argv.index("--") + 1]
bpy.ops.wm.save_as_mainfile(filepath=path)
