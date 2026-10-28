"""Grids shrinkwrapped onto a sphere and a cube, one per wrap method and mode,
for `tests/shrinkwrap_truth.rs`; Blender's own evaluated vertices are dumped
beside the file as `shrinkwrap_truth.txt`.

Targets, each with its own transform so the space conversion is exercised:
- "Ball": a smooth UV sphere (16 segments, 8 rings: quads and pole
  triangles), radius 1, at (0.3, -0.2, 0.1), rotated, scaled 1.2.
- "Box": a flat-shaded cube, 2 m, at (4, 0, 0), rotated 30 degrees in z.

Wrapped meshes: a 3 m grid of 9 x 9 vertices, tilted and lifted, under one
Shrinkwrap each:
- "Surface On": Nearest Surface Point, On Surface, offset 0.1.
- "Surface Inside" / "Surface Outside" / "Surface Outside Surface": the
  grid passes through the ball, offset 0.05.
- "Surface Above": Above Surface, offset 0.1 (the ball's smooth normals).
- "Box Above": Above Surface onto the flat cube, offset 0.1.
- "Vertex": Nearest Vertex, offset 0.05.
- "Project Z": Project along Z, both directions, offset 0.02.
- "Project Normal": Project along the vertex normals, both directions,
  back faces culled, limit 2.
- "Project Cull": from inside the ball along the normals, Above Surface
  offset 0.05, front faces culled and the culling inverted for the
  negative direction.
- "Group Invert": Nearest Surface with a vertex group (weights 0..1 across
  the grid), inverted.

Run: blender -b --python make_shrinkwrap.py -- shrinkwrap.blend"""
import bpy, bmesh, sys, os, math

bpy.ops.wm.read_factory_settings(use_empty=True)
sc = bpy.context.scene


def link(name, mesh):
    ob = bpy.data.objects.new(name, mesh)
    sc.collection.objects.link(ob)
    return ob


me = bpy.data.meshes.new("Ball")
bm = bmesh.new()
bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=8, radius=1.0)
for f in bm.faces:
    f.smooth = True
bm.to_mesh(me)
bm.free()
ball = link("Ball", me)
ball.location = (0.3, -0.2, 0.1)
ball.rotation_euler = (0.3, 0.2, 0.5)
ball.scale = (1.2, 1.2, 1.2)

me = bpy.data.meshes.new("Box")
bm = bmesh.new()
bmesh.ops.create_cube(bm, size=2.0)
for f in bm.faces:
    f.smooth = False
bm.to_mesh(me)
bm.free()
box = link("Box", me)
box.location = (4.0, 0.0, 0.0)
box.rotation_euler = (0.0, 0.0, math.radians(30))


def grid(name, z, target, x=0.0):
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_grid(bm, x_segments=8, y_segments=8, size=1.5)
    bm.to_mesh(me)
    bm.free()
    ob = link(name, me)
    ob.location = (x + 0.1, 0.05, z)
    ob.rotation_euler = (0.15, -0.1, 0.2)
    sw = ob.modifiers.new("Shrinkwrap", "SHRINKWRAP")
    sw.target = target
    return ob, sw


ob, sw = grid("Surface On", 1.6, ball)
sw.wrap_method, sw.wrap_mode, sw.offset = "NEAREST_SURFACEPOINT", "ON_SURFACE", 0.1
for mode in ("INSIDE", "OUTSIDE", "OUTSIDE_SURFACE"):
    ob, sw = grid("Surface " + mode.replace("_", " ").title(), 0.3, ball)
    sw.wrap_method, sw.wrap_mode, sw.offset = "NEAREST_SURFACEPOINT", mode, 0.05
ob, sw = grid("Surface Above", 1.6, ball)
sw.wrap_method, sw.wrap_mode, sw.offset = "NEAREST_SURFACEPOINT", "ABOVE_SURFACE", 0.1
ob, sw = grid("Box Above", 1.4, box, x=4.0)
sw.wrap_method, sw.wrap_mode, sw.offset = "NEAREST_SURFACEPOINT", "ABOVE_SURFACE", 0.1
ob, sw = grid("Vertex", 1.6, ball)
sw.wrap_method, sw.offset = "NEAREST_VERTEX", 0.05
ob, sw = grid("Project Z", 1.6, ball)
sw.wrap_method, sw.wrap_mode, sw.offset = "PROJECT", "ON_SURFACE", 0.02
sw.use_project_z, sw.use_negative_direction, sw.use_positive_direction = True, True, True
ob, sw = grid("Project Normal", 1.6, ball)
sw.wrap_method, sw.wrap_mode = "PROJECT", "ON_SURFACE"
sw.use_negative_direction, sw.use_positive_direction = True, True
sw.cull_face, sw.project_limit = "BACK", 2.0
ob, sw = grid("Project Cull", 0.3, ball)
sw.wrap_method, sw.wrap_mode, sw.offset = "PROJECT", "ABOVE_SURFACE", 0.05
sw.use_negative_direction, sw.use_positive_direction = True, True
sw.cull_face, sw.use_invert_cull = "FRONT", True
ob, sw = grid("Group Invert", 1.6, ball)
sw.wrap_method, sw.wrap_mode, sw.offset = "NEAREST_SURFACEPOINT", "ON_SURFACE", 0.0
g = ob.vertex_groups.new(name="wrap")
for v in ob.data.vertices:
    g.add([v.index], (v.co.x + 1.5) / 3.0, "REPLACE")
sw.vertex_group, sw.invert_vertex_group = "wrap", True

path = sys.argv[-1]
bpy.ops.wm.save_as_mainfile(filepath=path)

# Blender's own evaluated positions, object space, one line per object.
dg = bpy.context.evaluated_depsgraph_get()
with open(os.path.join(os.path.dirname(os.path.abspath(path)), "shrinkwrap_truth.txt"), "w") as out:
    for ob in sorted(sc.objects, key=lambda o: o.name):
        if not ob.modifiers:
            continue
        ev = ob.evaluated_get(dg).to_mesh()
        cos = " ".join(f"{c:.6f}" for v in ev.vertices for c in v.co)
        out.write(f"{ob.name}\t{len(ev.vertices)}\t{cos}\n")
