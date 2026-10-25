"""Write `demo-court.blend`, the animation demo: an arena in the LA28
palette (poppy, scarlet, bluebell, sagebrush), two teams of three rigged
players in real kits — sleeveless jersey and shorts as cloth over the body,
3D numbers and names front and back — playing a possession each way to a
2–2 scoreboard.

    pip install bpy==5.2.2 && python make_demo.py ../blend/demo-court.blend

It sits in `assets/source` beside the players it builds from, not in
`assets/blend`, which ships whole in the browser bundle.

Everything an evaluator plays is here: NLA strips — each player's ten
kinds of move (Run, Dribble, Pass, Catch, Jump Shot, Layup, Defend,
Contest, Pick Up, Celebrate) as actions laid back to back on one track,
the fingers on a track over it, the ball's flights on its own — pose and
object keys through them,
armature deform with smooth weights, and constant keys on the
scoreboard's digits. The court and boards are painted with numpy and
packed, the numbers are text converted to meshes, so the file opens the
same with nothing beside it. The 3D views are saved in Solid ▸ Texture with
shadows and cavity, the workbench look that shows the colours and the
depth at Solid's cost. Keys are sampled every other frame of the
360-frame game, thinned where a curve runs straight, rather than written
through cycle modifiers, so any evaluator that reads plain keyframes and
strips plays it.
"""

import math
import os
import sys

import bpy
import bmesh  # after bpy, which makes it importable
import numpy as np
from mathutils import Euler, Matrix, Quaternion, Vector

FRAMES = 360  # 15 s at 24 fps: one possession each way
OUT = sys.argv[-1]
FONT = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
scene.frame_start = 1
scene.frame_end = FRAMES
scene.render.fps = 24
scene.render.resolution_x = 1920
scene.render.resolution_y = 1080

# LA28's "Superbloom" colours, sRGB, plus the neutrals around them.
PAL = {
    "poppy": (0.96, 0.42, 0.20),
    "scarlet": (0.82, 0.12, 0.18),
    "bluebell": (0.38, 0.50, 0.90),
    "sage": (0.58, 0.68, 0.54),
    "sand": (0.96, 0.89, 0.76),
    "maple": (0.86, 0.66, 0.42),
    "ink": (0.09, 0.09, 0.13),
    "white": (0.97, 0.96, 0.93),
    "gold": (0.99, 0.78, 0.22),
}


def lin(c):
    """sRGB → scene-linear, for material and viewport colours."""
    return tuple(x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4 for x in c)


def material(name, color, rough=0.6, metal=0.0, image=None, emit=0.0):
    """A Principled material; `color` (sRGB) is also the Solid colour.
    `image` drives Base Color (and Emission, when `emit`)."""
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nodes = m.node_tree.nodes
    bsdf = nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (*lin(color), 1.0)
    bsdf.inputs["Roughness"].default_value = rough
    bsdf.inputs["Metallic"].default_value = metal
    m.diffuse_color = (*lin(color), 1.0)
    m.roughness = rough
    if image is not None:
        tex = nodes.new("ShaderNodeTexImage")
        tex.image = image
        tex.location = (-320, 300)
        m.node_tree.links.new(tex.outputs["Color"], bsdf.inputs["Base Color"])
        if emit:
            m.node_tree.links.new(tex.outputs["Color"], bsdf.inputs["Emission Color"])
    if emit:
        if image is None:
            bsdf.inputs["Emission Color"].default_value = (*lin(color), 1.0)
        bsdf.inputs["Emission Strength"].default_value = emit
    return m


def packed_image(name, rgb):
    """Pack an (h, w, 3) sRGB array, row 0 at the top, into the file."""
    h, w, _ = rgb.shape
    img = bpy.data.images.new(name, w, h, alpha=False)
    rgba = np.ones((h, w, 4), dtype=np.float32)
    rgba[..., :3] = np.clip(rgb, 0.0, 1.0)
    img.pixels.foreach_set(rgba[::-1].ravel())  # Blender's rows run bottom-up
    img.pack()
    return img


def srgb(c):
    return np.array(c, dtype=np.float32)


def link(obj):
    scene.collection.objects.link(obj)
    return obj


def activate(obj):
    for o in bpy.context.view_layer.objects:
        o.select_set(False)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)


def mesh_object(name, mesh, mats=(), loc=(0, 0, 0)):
    obj = link(bpy.data.objects.new(name, mesh))
    obj.location = loc
    for m in mats:
        mesh.materials.append(m)
    return obj


def box(name, mat, centre, size):
    """An axis-aligned box mesh object (`size` full extents)."""
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        v.co = Vector((v.co.x * size[0], v.co.y * size[1], v.co.z * size[2]))
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    return mesh_object(name, mesh, [mat], centre)


def primitive(op, name, mat=None, loc=(0, 0, 0), scale=None, rot=None, **kw):
    op(location=loc, **kw)
    obj = bpy.context.active_object
    obj.name = name
    obj.data.name = name
    if scale:
        obj.scale = scale
    if rot:
        obj.rotation_euler = rot
    if mat is not None:
        obj.data.materials.append(mat)
    return obj


# ── The floor: court and apron, 40 px a metre ─────────────────────────────
COURT_X, COURT_Y = 14.0, 7.5  # FIBA, 28 × 15 m
FLOOR_X, FLOOR_Y = 16.5, 10.0  # the apron out to the boards
PX = 40


def squircle(dx, dy, n=3.2, wide=1.15):
    """The phiegg ring's distance: a superellipse of power `n` — a circle
    at 2, a square as it grows — `wide` times longer across x than y, so
    its rings are softly squarish ovals rather than circles."""
    return (np.abs(dx / wide) ** n + np.abs(dy) ** n) ** (1.0 / n)


def bloom(mx, my, cx, cy, rings, colors, width):
    """LA28-style concentric rings at (cx, cy), softly squarish ovals: a
    mask per colour."""
    r = squircle(mx - cx, my - cy)
    out = []
    for i, (radius, col) in enumerate(zip(rings, colors)):
        out.append((np.abs(r - radius) < width / 2, col))
    return out


def floor_texture():
    w, h = int(2 * FLOOR_X * PX), int(2 * FLOOR_Y * PX)
    rng = np.random.default_rng(7)
    ys, xs = np.mgrid[0:h, 0:w].astype(np.float32)
    mx = (xs + 0.5) / PX - FLOOR_X
    my = FLOOR_Y - (ys + 0.5) / PX
    on_court = (np.abs(mx) <= COURT_X) & (np.abs(my) <= COURT_Y)

    # The apron: sagebrush, with bluebell and poppy arcs swept across it.
    img = np.empty((h, w, 3), np.float32)
    img[:] = srgb(PAL["sage"])
    sweep = np.hypot(mx * 0.9, my - 30.0)
    band = (np.floor(sweep / 1.2) % 3).astype(int)
    for k, col in enumerate((PAL["sage"], PAL["bluebell"], PAL["sand"])):
        img[(band == k) & ~on_court] = srgb(col) * 0.92 + 0.04

    # The court: maple boards along the length, joints staggered.
    plank_w = 0.18 * PX
    plank = (ys // plank_w).astype(int)
    tone = rng.uniform(-0.04, 0.04, plank.max() + 1)[plank]
    stagger = rng.uniform(0, 3 * PX, plank.max() + 1)[plank]
    joint = ((xs + stagger) % (3.0 * PX)) < 1.0
    seam = (ys % plank_w) < 1.0
    grain = 0.02 * np.sin(xs * 0.07 + 6.0 * np.sin(ys * 0.05 + plank))  # no per-pixel noise: it bloats the packed PNG
    wood = srgb(PAL["maple"])[None, None, :] * (1.0 + tone + grain)[..., None]
    wood[seam | joint] *= 0.88
    img[on_court] = wood[on_court]

    lw = 0.05
    line = np.zeros((h, w), bool)
    line |= on_court & ((np.abs(np.abs(mx) - (COURT_X - lw)) < lw) | (np.abs(np.abs(my) - (COURT_Y - lw)) < lw))
    line |= on_court & (np.abs(mx) < lw / 2 + 0.01)
    paint = np.zeros((h, w, 3), np.float32)
    painted = np.zeros((h, w), bool)
    for side, key_col in ((-1, PAL["scarlet"]), (1, PAL["bluebell"])):
        hoop = side * (COURT_X - 1.575)
        ax = side * mx
        key = (ax > COURT_X - 5.8) & (np.abs(my) < 2.45) & on_court
        paint[key] = srgb(key_col)
        painted |= key
        edge = key & ~((ax > COURT_X - 5.8 + lw) & (np.abs(my) < 2.45 - lw) & (ax < COURT_X - lw))
        line |= edge
        line |= (np.abs(np.hypot(mx - side * (COURT_X - 5.8), my) - 1.8) < lw / 2 + 0.01) & (ax < COURT_X - 5.8)
        d = np.hypot(mx - hoop, my)
        arc = (np.abs(d - 6.75) < lw / 2 + 0.01) & (np.abs(my) < COURT_Y - 0.9) & (ax > 0) & (ax < COURT_X)
        corner = (np.abs(np.abs(my) - (COURT_Y - 0.9)) < lw / 2 + 0.01) & (ax > COURT_X - 1.575) & (ax < COURT_X)
        line |= arc | corner
    # The centre: a bloom of rings in the four colours.
    for mask, col in bloom(mx, my, 0.0, 0.0, (0.6, 1.0, 1.4, 1.8), (PAL["gold"], PAL["poppy"], PAL["scarlet"], PAL["bluebell"]), 0.36):
        paint[mask] = srgb(col)
        painted |= mask
    img[painted] = img[painted] * 0.25 + paint[painted] * 0.75
    line |= np.abs(squircle(mx, my) - 2.0) < lw / 2 + 0.01
    img[line] = srgb(PAL["white"])
    return packed_image("Arena Floor", img)


floor_mat = material("Arena Floor", PAL["maple"], rough=0.3, image=floor_texture())
floor = primitive(bpy.ops.mesh.primitive_plane_add, "Court", floor_mat, size=1, scale=(2 * FLOOR_X, 2 * FLOOR_Y, 1))
bpy.ops.object.transform_apply(scale=True)

# ── Hoops ─────────────────────────────────────────────────────────────────
steel = material("Hoop Steel", PAL["ink"], rough=0.35, metal=0.6)
pad = material("Hoop Padding", PAL["poppy"], rough=0.7)
glass = material("Backboard", PAL["white"], rough=0.15)
rim_mat = material("Rim", PAL["poppy"], rough=0.35, metal=0.4)
for side, x in (("L", -1.0), ("R", 1.0)):
    toward = -x
    base_x = x * (COURT_X + 1.6)
    box(f"Hoop Base.{side}", pad, (base_x, 0, 0.45), (1.2, 1.4, 0.9))
    primitive(bpy.ops.mesh.primitive_cylinder_add, f"Hoop Pole.{side}", steel, loc=(base_x, 0, 2.0), radius=0.09, depth=2.4)
    box(f"Hoop Arm.{side}", steel, ((base_x + x * (COURT_X - 1.2)) / 2, 0, 3.1), (abs(base_x - x * (COURT_X - 1.2)), 0.12, 0.12))
    box(f"Backboard.{side}", glass, (x * (COURT_X - 1.2), 0, 3.35), (0.04, 1.8, 1.05))
    primitive(
        bpy.ops.mesh.primitive_torus_add,
        f"Rim.{side}",
        rim_mat,
        loc=(x * (COURT_X - 1.575), 0, 3.05),
        major_radius=0.23,
        minor_radius=0.018,
    )

# ── LED boards and stands ─────────────────────────────────────────────────


def board_texture():
    """A strip of LA28 blooms: concentric squarish ovals on ink."""
    w, h = 1024, 128
    ys, xs = np.mgrid[0:h, 0:w].astype(np.float32)
    img = np.empty((h, w, 3), np.float32)
    img[:] = srgb(PAL["ink"])
    cols = [PAL["poppy"], PAL["bluebell"], PAL["sage"], PAL["scarlet"], PAL["gold"]]
    for k in range(8):
        cx, cy = 64 + k * 128, 64 if k % 2 else 128
        r = squircle(xs - cx, ys - cy)
        for j in range(4):
            img[np.abs(r - (16 + j * 18)) < 6.0] = srgb(cols[(k + j) % len(cols)])
    return packed_image("LED Boards", img)


board_mat = material("LED Boards", PAL["poppy"], rough=0.4, image=board_texture(), emit=1.5)


def uv_box(name, mat, centre, size, u_repeat):
    """A box whose texture runs along its length `u_repeat` times and up
    its height once, on every face."""
    obj = box(name, mat, centre, size)
    uv = obj.data.uv_layers.new(name="UVMap")
    along = 0 if size[0] >= size[1] else 1
    for poly in obj.data.polygons:
        for li in poly.loop_indices:
            co = obj.data.vertices[obj.data.loops[li].vertex_index].co
            uv.data[li].uv = ((co[along] / size[along] + 0.5) * u_repeat, co.z / size[2] + 0.5)
    return obj


for side in (-1, 1):
    uv_box(f"LED Board.{'N' if side > 0 else 'S'}", board_mat, (0, side * (FLOOR_Y - 0.4), 0.45), (2 * COURT_X + 2, 0.12, 0.9), 5)
    uv_box(f"LED Board.{'E' if side > 0 else 'W'}", board_mat, (side * (FLOOR_X - 0.2), 0, 0.45), (0.12, 2 * COURT_Y + 1, 0.9), 3)

seat_cols = [PAL["poppy"], PAL["sand"], PAL["bluebell"], PAL["sage"], PAL["scarlet"], PAL["sand"]]
seat_mats = [material(f"Seats {k}", c, rough=0.8) for k, c in enumerate(seat_cols)]
TIERS, BACK = 8, 0.85  # terraces, and how deep each is


def tier_rise(i):
    return 0.45 * (i + 1)


def corner_tier(name, mat, i, sx, sy, segments=12):
    """Terrace `i` turning the corner at (sx·FLOOR_X, sy·FLOOR_Y): a quarter
    ring meeting the straight tiers either side, its top, front and back."""
    r0 = 0.6 + i * BACK - BACK / 2
    r1, rise = r0 + BACK, tier_rise(i)
    cx, cy = sx * FLOOR_X, sy * FLOOR_Y
    bm = bmesh.new()
    ring = []
    for k in range(segments + 1):
        a = (math.pi / 2) * k / segments
        d = Vector((sx * math.cos(a), sy * math.sin(a), 0.0))
        ring.append([bm.verts.new((cx + d.x * r, cy + d.y * r, z)) for r, z in ((r0, 0.0), (r0, rise), (r1, rise), (r1, 0.0))])
    for a, b in zip(ring, ring[1:]):
        for j in range(3):
            face = (a[j], a[j + 1], b[j + 1], b[j])
            bm.faces.new(face if sx * sy > 0 else face[::-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    return mesh_object(name, mesh, [mat])


for i in range(TIERS):
    rise = tier_rise(i)
    mat = seat_mats[i % len(seat_mats)]
    for side in (-1, 1):
        y = side * (FLOOR_Y + 0.6 + i * BACK)
        box(f"Stand.{'N' if side > 0 else 'S'}.{i}", mat, (0, y, rise / 2), (2 * FLOOR_X, BACK, rise))
        x = side * (FLOOR_X + 0.6 + i * BACK)
        box(f"Stand.{'E' if side > 0 else 'W'}.{i}", mat, (x, 0, rise / 2), (BACK, 2 * FLOOR_Y, rise))
    for sx in (-1, 1):
        for sy in (-1, 1):
            corner_tier(f"Stand.{'N' if sy > 0 else 'S'}{'E' if sx > 0 else 'W'}.{i}", mat, i, sx, sy)

# Every terrace carries a row of seats, each a nail of a thing — a slim
# hexagonal stem, a softly squarish oval pan, and a back shaped like the
# phiegg mark (puikit's `brand/marks.tsx`): full and squarish at the
# shoulders, narrowing like a rocket's fairing to a third of its width at
# the top, cut in ten soft facets — set back on the step and facing
# the court. A seat's place and heading come from the terrace it is on:
# along the straight tiers and round the corners alike.
SEAT_PITCH, SEAT_H = 0.55, 0.42


def seat_places():
    """(tier, position on the step, heading toward the court) of every seat."""
    out = []
    for i in range(TIERS):
        rise = tier_rise(i)
        r = 0.6 + i * BACK + 0.12  # a little behind the step's middle
        n = int(2 * FLOOR_X / SEAT_PITCH)
        for side, heading in ((1, 0.0), (-1, math.pi)):
            for k in range(n):
                out.append((i, Vector(((k + 0.5) * SEAT_PITCH - FLOOR_X, side * (FLOOR_Y + r), rise)), heading))
        n = int(2 * FLOOR_Y / SEAT_PITCH)
        for side, heading in ((1, -math.pi / 2), (-1, math.pi / 2)):
            for k in range(n):
                out.append((i, Vector((side * (FLOOR_X + r), (k + 0.5) * SEAT_PITCH - FLOOR_Y, rise)), heading))
        n = max(1, int((math.pi / 2) * r / SEAT_PITCH))
        for sx in (-1, 1):
            for sy in (-1, 1):
                for k in range(n):
                    a = (math.pi / 2) * (k + 0.5) / n
                    d = Vector((sx * math.cos(a), sy * math.sin(a), 0.0))
                    out.append((i, Vector((sx * FLOOR_X, sy * FLOOR_Y, rise)) + d * r, math.atan2(-d.x, d.y)))
    return out


def phiegg(half_w, half_h, top=1 / 3, n=2.6, points=10):
    """The outline, (across, up) about its centre: a superellipse of power
    `n` (softly squarish) whose upper half narrows to `top` of its width.
    `top` 1 is the plain squarish oval."""
    out = []
    for k in range(points):
        a = 2 * math.pi * k / points - math.pi / 2  # from the bottom, round
        c, s = math.cos(a), math.sin(a)
        up = math.copysign(abs(s) ** (2 / n), s)
        taper = 1.0 - (1.0 - top) * max(0.0, up) ** 1.3
        out.append((math.copysign(abs(c) ** (2 / n), c) * half_w * taper, up * half_h))
    return out


def prism(bm, outline, place, depth):
    """`outline` (x, z at y 0) extruded `depth` along +y, placed by `place`."""
    rings = [[bm.verts.new(place @ Vector((x, y, z))) for x, z in outline] for y in (0.0, depth)]
    bm.faces.new(rings[0][::-1])
    bm.faces.new(rings[1])
    m = len(outline)
    for k in range(m):
        bm.faces.new((rings[0][k], rings[0][(k + 1) % m], rings[1][(k + 1) % m], rings[1][k]))


SEAT_BACK = phiegg(0.19, 0.21)  # 0.38 m across at the shoulders, 0.42 tall
SEAT_PAN = phiegg(0.21, 0.19, top=1.0, n=3.0)  # 0.42 × 0.38, softly square


def seat_shape(bm, at, heading):
    """One seat into `bm`: a hexagonal stem, a squarish oval pan, a phiegg
    back leaning a little away from the court."""
    turn = Matrix.Translation(at) @ Matrix.Rotation(heading, 4, "Z")
    bmesh.ops.create_cone(bm, cap_ends=True, segments=6, radius1=0.035, radius2=0.03, depth=SEAT_H - 0.04, matrix=turn @ Matrix.Translation((0, 0, (SEAT_H - 0.04) / 2)))
    # The pan's outline lies flat: (x, z) of the outline becomes (x, y).
    flat = turn @ Matrix.Translation((0, 0, SEAT_H - 0.04)) @ Matrix.Rotation(math.pi / 2, 4, "X")
    prism(bm, SEAT_PAN, flat, 0.05)
    lean = Matrix.Translation((0, 0.17, SEAT_H + 0.02)) @ Matrix.Rotation(math.radians(-10), 4, "X") @ Matrix.Translation((0, 0, 0.21))
    prism(bm, SEAT_BACK, turn @ lean, 0.035)


SEATS = seat_places()
seat_shell = [material(f"Seat Shell {k}", c, rough=0.45) for k, c in enumerate([PAL["white"], PAL["scarlet"], PAL["ink"], PAL["bluebell"], PAL["gold"], PAL["sage"]])]
for i in range(TIERS):
    bm = bmesh.new()
    for tier, at, heading in SEATS:
        if tier == i:
            seat_shape(bm, at, heading)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    mesh = bpy.data.meshes.new(f"Seats.{i}")
    bm.to_mesh(mesh)
    bm.free()
    mesh_object(f"Seats.{i}", mesh, [seat_shell[(i + 3) % len(seat_shell)]])

# ── The basketball ────────────────────────────────────────────────────────


def ball_texture():
    """Equirectangular: pebbled orange leather and a basketball's eight-panel
    seams — two great circles crossing at right angles (the equator z = 0
    and the meridian x = 0) and the two curved seams, each a circle round
    the meridian's axis (±x) that crosses the equator and never the
    meridian: the two great circles make four lunes, and each curved seam
    halves the two lunes on its side — eight panels."""
    w, h = 512, 256
    u = (np.arange(w) + 0.5) / w
    v = (np.arange(h) + 0.5) / h
    uu, vv = np.meshgrid(u, v)
    theta, phi = np.pi * vv, 2 * np.pi * uu  # row 0 is the north pole
    d = np.stack([np.sin(theta) * np.cos(phi), np.sin(theta) * np.sin(phi), np.cos(theta)], axis=-1)
    seam_w = 0.022  # radians either side: a groove some 5 mm wide
    seam = (np.abs(d[..., 2]) < seam_w) | (np.abs(d[..., 0]) < seam_w)
    for sign in (1.0, -1.0):
        angle = np.arccos(np.clip(sign * d[..., 0], -1.0, 1.0))
        seam |= np.abs(angle - 0.92) < seam_w
    # The pebble: a fine regular grain, darker in its pits (a pattern, not
    # noise, so the packed image stays small).
    grain = np.cos(phi * 96) * np.cos(theta * 96)
    img = srgb((0.84, 0.38, 0.11))[None, None, :] * (1.0 - 0.07 * (grain > 0.55))[..., None]
    img[seam] = srgb((0.06, 0.04, 0.03))
    return packed_image("Basketball Leather", img)


BALL_LEATHER = ball_texture()
ball_mat = material("Basketball", (0.86, 0.40, 0.12), rough=0.55, image=BALL_LEATHER)
ball = primitive(bpy.ops.mesh.primitive_uv_sphere_add, "Basketball", ball_mat, loc=(0, 0, 0.6), radius=0.12, segments=32, ring_count=16)
bpy.ops.object.shade_smooth()

# ── The players: Female-Epic-01, in kit ──────────────────────────────────
# `assets/source/epic01-player.blend` (written by
# `assets/source/extract_mannequin.py epic01` from the Female-Epic-01
# character, and kept out of `assets/blend`, which the browser build embeds
# whole) holds her body, eyes, brows, lashes, nose and hair, and a plain
# armature of the Rigify rig's 204 deform bones. Every player shares those
# meshes; each team has its own vest and shorts, cut from the body; numbers
# and names are text converted to mesh and wrapped onto the vest.
PLAYER = os.path.join(os.path.dirname(os.path.abspath(__file__)), "epic01-player.blend")
with bpy.data.libraries.load(PLAYER) as (src, dst):
    dst.objects = [n for n in src.objects if n.startswith("Player ")]
TEMPLATE = {ob.name.removeprefix("Player "): ob for ob in dst.objects}
REST = TEMPLATE["Rig"].data  # the deform skeleton every player's rig copies
BODY_MESH = TEMPLATE["Body"].data
GROUPS = [g.name for g in TEMPLATE["Body"].vertex_groups]

# Her proportions, from her skeleton: where the kit starts and ends.
_head = lambda bone: REST.bones[bone].head_local.z
NECK_Z, WAIST_Z = _head("DEF-spine.004"), _head("DEF-spine.001")
HIP_Z, KNEE_Z = _head("DEF-thigh.L"), _head("DEF-shin.L")
VEST_NECK, VEST_HEM = NECK_Z - 0.01, WAIST_Z - 0.04  # the vest hangs over the waistband
WAISTBAND, SHORTS_HEM = WAIST_Z + 0.03, KNEE_Z + 0.45 * (HIP_Z - KNEE_Z)
PANEL_X = 0.105  # the vest's side panels
STRAP = (0.06, 0.115)  # the shoulder straps, either side of the neck
ARM = ("DEF-upper_arm", "DEF-forearm", "DEF-hand", "DEF-palm", "DEF-f_", "DEF-thumb")


def text_mesh(name, txt, size, loc, rot, mat):
    """`txt` as a flat mesh in the bold face, densified so it can be
    wrapped onto the vest."""
    cu = bpy.data.curves.new(name, "FONT")
    cu.body = txt
    cu.font = bpy.data.fonts.load(FONT, check_existing=True)
    cu.size = size
    cu.align_x = "CENTER"
    cu.align_y = "CENTER"
    cu.resolution_u = 6
    obj = link(bpy.data.objects.new(name, cu))
    obj.location = loc
    obj.rotation_euler = rot
    activate(obj)
    bpy.ops.object.convert(target="MESH")
    obj = bpy.context.active_object
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    bmesh.ops.subdivide_edges(bm, edges=bm.edges[:], cuts=1, use_grid_fill=True)
    bm.to_mesh(obj.data)
    bm.free()
    obj.data.materials.clear()
    obj.data.materials.append(mat)
    return obj


def surface_y(mesh, z, back):
    """How far the vest stands out on the centre line at height `z`:
    behind (+Y, she faces −Y) or in front."""
    ys = [v.co.y for v in mesh.vertices if abs(v.co.x) < 0.04 and abs(v.co.z - z) < 0.04]
    return max(ys) if back else min(ys)


def subdivided(mesh):
    """`mesh` one Catmull-Clark level finer, its weights carried along."""
    ob = link(bpy.data.objects.new("_smooth", mesh))
    ob.modifiers.new("Smooth", "SUBSURF").levels = 1
    depsgraph = bpy.context.evaluated_depsgraph_get()
    out = bpy.data.meshes.new_from_object(ob.evaluated_get(depsgraph), preserve_all_data_layers=True, depsgraph=depsgraph)
    bpy.data.objects.remove(ob)
    bpy.data.meshes.remove(mesh)
    return out


def strongest(mesh, n=4):
    """Keep each vertex's `n` strongest weights over 0.01, normalised, as a
    game rig does. The source paints every vertex with trace weights for
    dozens of face and finger bones (42 a vertex on her body, 82% under
    0.001); the Subdivision ahead of the Armature spreads them further, and
    every one is skinned every frame — 15 million influences across the six
    players, a second to pose the scene."""
    bm = bmesh.new()
    bm.from_mesh(mesh)
    deform = bm.verts.layers.deform.active
    if deform is not None:
        for v in bm.verts:
            w = sorted(v[deform].items(), key=lambda gw: -gw[1])
            top = [(g, x) for g, x in w[:n] if x > 0.01]
            total = sum(x for _, x in top) or 1.0
            v[deform].clear()
            for g, x in top:
                v[deform][g] = x / total
    bm.to_mesh(mesh)
    bm.free()


def garment(name, keep, offset, cuts_z, cuts_x=(), smooth=True):
    """A garment cut from the body: the faces `keep(centre, arm_share)`
    wants, the body cut clean along `cuts_z` / `cuts_x` first so its edges
    are straight lines, stood `offset(co)` off the skin, then (`smooth`)
    subdivided once to match the body's smoothing. Its vertices keep the
    weights of the body beneath, so it never parts from it."""
    out = BODY_MESH.copy()  # a copy keeps the group names, which live on the mesh
    out.name = name
    out.materials.clear()
    bm = bmesh.new()
    bm.from_mesh(out)
    deform = bm.verts.layers.deform.active
    arm_ids = {i for i, n in enumerate(GROUPS) if n.startswith(ARM)}
    for z in cuts_z:
        bmesh.ops.bisect_plane(bm, geom=bm.verts[:] + bm.edges[:] + bm.faces[:], plane_co=(0, 0, z), plane_no=(0, 0, 1))
    for x in cuts_x:
        bmesh.ops.bisect_plane(bm, geom=bm.verts[:] + bm.edges[:] + bm.faces[:], plane_co=(x, 0, 0), plane_no=(1, 0, 0))

    def arm_share(face):
        total = part = 0.0
        for v in face.verts:
            for g, w in v[deform].items():
                total += w
                if g in arm_ids:
                    part += w
        return part / max(total, 1e-6)

    kept = {f for f in bm.faces if keep(f.calc_center_median(), arm_share(f))}
    bmesh.ops.delete(bm, geom=[f for f in bm.faces if f not in kept], context="FACES")
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")
    bm.normal_update()  # the bisects leave the new vertices without normals
    for v in bm.verts:
        v.co += v.normal * offset(v.co)
    bm.to_mesh(out)
    bm.free()
    if smooth:
        out = subdivided(out)
        out.name = name
    # Subdivision spreads every corner's weights over its neighbours.
    strongest(out)
    for p in out.polygons:
        p.use_smooth = True
    return out


def vest_keeps(c, arm):
    """The trunk between the hem and the neckline (the T-pose's hands lie in
    the same band of height), and a strap over each shoulder."""
    trunk = VEST_HEM < c.z < VEST_NECK and abs(c.x) < 0.28
    strap = VEST_NECK <= c.z < VEST_NECK + 0.08 and STRAP[0] < abs(c.x) < STRAP[1]
    return (trunk or strap) and arm < 0.35


def shorts_keeps(c, arm):
    return SHORTS_HEM < c.z < WAISTBAND and abs(c.x) < 0.28 and arm < 0.35


def shorts_offset(co):
    """Snug at the waistband, flaring to 4 cm at the hem."""
    t = (WAISTBAND - 0.08 - co.z) / max(WAISTBAND - 0.08 - SHORTS_HEM, 1e-6)
    return 0.011 + 0.04 * max(0.0, min(1.0, t)) ** 1.5


VEST = garment("Vest", vest_keeps, lambda co: 0.016, (VEST_HEM, VEST_NECK), (-PANEL_X, PANEL_X, -STRAP[0], STRAP[0], -STRAP[1], STRAP[1]))
SHORTS = garment("Shorts", shorts_keeps, shorts_offset, (SHORTS_HEM, WAISTBAND))
# Basketball shoes: a high-top shell over the foot up to the ankle, stood
# off the skin more toward the sole so the sole has some depth. The game's
# ground contact stands the players on it (`SOLE`). Not subdivided: the
# foot is the body's densest part (its toes), and a smoothed shell would
# outweigh the whole body for a centimetre of curvature nobody sees; the
# smoothed foot beneath only draws further in.
SHOE_TOP = 0.13
SNEAKER = garment("Sneaker", lambda c, arm: c.z < SHOE_TOP, lambda co: 0.011 + 0.013 * max(0.0, (0.03 - co.z) / 0.03), (SHOE_TOP,), smooth=False)
# The kit is cut from the body as painted; the players wear it thinned.
for role, ob in TEMPLATE.items():
    if ob.type == "MESH":
        strongest(ob.data)


def team_kit(team):
    """The team's vest and shorts in its colours: the vest's side panels and
    the shorts' hem and side stripes in the trim."""
    jersey = material(f"{team['name']} Jersey", team["jersey"], rough=0.55)
    trim = material(f"{team['name']} Trim", team["trim"], rough=0.55)
    shorts = material(f"{team['name']} Shorts", team["shorts"], rough=0.6)
    top = VEST.copy()
    top.name = f"{team['name']} Jersey"
    top.materials.append(jersey)
    top.materials.append(trim)
    for p in top.polygons:
        p.material_index = 1 if abs(p.center.x) > PANEL_X else 0
    low = SHORTS.copy()
    low.name = f"{team['name']} Shorts"
    low.materials.append(shorts)
    low.materials.append(trim)
    for p in low.polygons:
        p.material_index = 1 if p.center.z < SHORTS_HEM + 0.035 else 0
    # The sneakers: a white upper on a dark sole, a band of the trim round
    # the foot and the jersey's colour at the collar.
    shoe = SNEAKER.copy()
    shoe.name = f"{team['name']} Sneakers"
    for m in (material("Sneaker Upper", PAL["white"], rough=0.45), material("Sneaker Sole", PAL["ink"], rough=0.7), trim, jersey):
        shoe.materials.append(m)
    for p in shoe.polygons:
        z = p.center.z
        p.material_index = 1 if z < 0.028 else 2 if 0.045 < z < 0.07 else 3 if z > SHOE_TOP - 0.025 else 0
    return top, low, shoe


# Her look, shared by every player: skin, eyes, and dark hair.
SKIN = material("Skin", (0.62, 0.43, 0.31), rough=0.42)
LOOK = {"Body": SKIN, "Nose": SKIN}
HAIR = material("Hair", (0.10, 0.07, 0.05), rough=0.5)
EYE = [material("Sclera", (0.93, 0.91, 0.88), rough=0.1), material("Iris", (0.30, 0.17, 0.08), rough=0.1), material("Pupil", (0.02, 0.02, 0.02), rough=0.1)]
for role, ob in TEMPLATE.items():
    if role == "Rig":
        continue
    if role.startswith("Eye."):
        # The eyeball by where each face looks from its centre: the cap
        # that faces forward (−Y) is the pupil, the ring round it the iris,
        # the rest white.
        for m in EYE:
            ob.data.materials.append(m)
        centre = sum((v.co for v in ob.data.vertices), Vector()) / len(ob.data.vertices)
        for p in ob.data.polygons:
            look = (p.center - centre).normalized().dot(Vector((0, -1, 0)))
            p.material_index = 2 if look > 0.975 else 1 if look > 0.90 else 0
    else:
        ob.data.materials.append(LOOK.get(role, HAIR))
    for p in ob.data.polygons:
        p.use_smooth = True


def skinned(name, mesh, rig, smooth):
    """An object of `mesh` deformed by `rig` — a Subdivision ahead of the
    Armature when the source smoothed it (evaluated once at load). The
    vertex groups come with the mesh (their names live on it)."""
    ob = link(bpy.data.objects.new(name, mesh))
    ob.parent = rig
    if smooth:
        ob.modifiers.new("Smooth", "SUBSURF").levels = 1
    ob.modifiers.new("Armature", "ARMATURE").object = rig
    return ob


def player(name, team, number, kit):
    """One player: a rig of the deform skeleton, her body, face and hair,
    the team's kit, and the number on the back and front with the name
    above it."""
    rig = link(bpy.data.objects.new(f"{name} Rig", REST.copy()))
    rig.data.name = f"{name} Rig"
    rig.data.display_type = "STICK"
    for role, ob in TEMPLATE.items():
        if role != "Rig":
            skinned(f"{name} {role}", ob.data, rig, ob.get("smooth", 0))
    skinned(f"{name} Jersey", kit[0], rig, False)
    skinned(f"{name} Shorts", kit[1], rig, False)
    skinned(f"{name} Sneakers", kit[2], rig, False)

    ink = bpy.data.materials.get(f"{team['name']} Numbers") or material(f"{team['name']} Numbers", team["numbers"], rough=0.5)
    b_no, b_name, f_no = NECK_Z - 0.20, NECK_Z - 0.07, NECK_Z - 0.13
    back, front = surface_y(kit[0], b_no + 0.02, True), surface_y(kit[0], f_no, False)
    # Facing −Y: the back faces +Y, read from behind; the front faces −Y.
    pieces = [
        text_mesh(f"{name} No.B", str(number), 0.14, (0, back + 0.02, b_no), (math.pi / 2, 0, math.pi), ink),
        text_mesh(f"{name} Name", name.upper(), 0.042, (0, back + 0.02, b_name), (math.pi / 2, 0, math.pi), ink),
        text_mesh(f"{name} No.F", str(number), 0.08, (0, front - 0.02, f_no), (math.pi / 2, 0, 0), ink),
    ]
    target = link(bpy.data.objects.new("_vest", kit[0]))
    for piece in pieces:
        wrap = piece.modifiers.new("Wrap", "SHRINKWRAP")
        wrap.target = target
        wrap.wrap_method = "NEAREST_SURFACEPOINT"
        wrap.wrap_mode = "OUTSIDE_SURFACE"
        wrap.offset = 0.003
        activate(piece)
        bpy.ops.object.modifier_apply(modifier="Wrap")
    bpy.data.objects.remove(target)
    activate(pieces[0])
    for piece in pieces[1:]:
        piece.select_set(True)
    bpy.ops.object.join()
    numbers = bpy.context.active_object
    numbers.name = numbers.data.name = f"{name} Numbers"
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    for p in numbers.data.polygons:
        p.use_smooth = True
    numbers.vertex_groups.new(name="DEF-spine.003").add(list(range(len(numbers.data.vertices))), 1.0, "REPLACE")
    numbers.parent = rig
    numbers.modifiers.new("Armature", "ARMATURE").object = rig
    return rig


# ── The game ──────────────────────────────────────────────────────────────
# One possession each way, played to the rules the Olympics teach: the
# ball moves by the dribble (one hand, bounced off the floor) or the pass;
# a player gathering it may take two steps — a layup's "easy steps" — and
# no more; a basket inside the arc is two points.
#
# Home attack the west basket. Ayanda brings the ball up and passes to
# Buhle, who takes two dribbles and makes a jump shot over Lerato: 2–0.
# Dumi gathers the ball under the basket, dribbles the length of the floor
# and passes to Lerato, who dribbles, gathers and lays it in on two
# steps: 2–2. A defender keeps between their player and the basket,
# facing the ball; a player without the ball runs to get open.
#
# Each move is an action of its own — Run, Dribble, Pass, Catch, Jump
# Shot, Layup, Defend, Contest, Pick Up, Celebrate — laid as strips on its
# player's NLA track, the strips back to back; the ball's flights are
# strips on its own track; the scoreboard's digits switch on constant keys.
# Poses are authored as rotations about the world's axes at rest (the
# player faces −Y, up is +Z), each bone's carried by its parent's.
FPS = scene.render.fps
X, Y, Z = (1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)
HOOP = {-1: Vector((-(COURT_X - 1.575), 0.0, 3.05)), 1: Vector((COURT_X - 1.575, 0.0, 3.05))}
BALL_R = 0.12
FINGERS = {side: [b.name for b in REST.bones if b.name.endswith(f".{side}") and b.name.startswith(("DEF-f_", "DEF-thumb"))] for side in ("L", "R")}


def q(axis, angle):
    return Quaternion(Vector(axis), angle)


def smooth(x):
    x = max(0.0, min(1.0, x))
    return x * x * (3 - 2 * x)


def ramp(t, a, b):
    """0 before `a`, 1 after `b`, eased between."""
    return smooth((t - a) / (b - a)) if b > a else float(t >= b)


def mix(a, b, w):
    return a + (b - a) * w


# ── Pose pieces ──
def arm(d, side, lower, swing, flex, curl=0.55, wrist=0.0):
    """An arm `lower`ed from the T (about Y; 1.25 hangs it), `swing`
    about X (negative forward and up), the elbow flexed; the clavicle
    takes a share of the swing and rises when the arm goes overhead."""
    sign = 1.0 if side == "L" else -1.0
    overhead = ramp(-swing, 1.4, 2.7)
    clav = max(-0.3, min(0.3, 0.15 * swing))
    d[f"DEF-shoulder.{side}"] = q(X, clav) @ q(Y, sign * (0.18 * max(0.0, min(1.0, lower / 1.25)) - 0.30 * overhead))
    d[f"DEF-upper_arm.{side}"] = q(X, swing - clav) @ q(Y, sign * (lower - 0.18))
    d[f"DEF-forearm.{side}"] = q(Z, -sign * flex)
    d[f"DEF-hand.{side}"] = q(Y, sign * wrist)
    for finger in FINGERS[side]:
        d[finger] = q(Y, sign * (0.45 * curl if "thumb" in finger else curl))


def leg(d, side, thigh, knee, foot=0.0, out=0.0):
    """A leg: the thigh swung forward by `thigh` (and `out` to the side),
    the knee folded by `knee`, the ankle by `foot`."""
    sign = 1.0 if side == "L" else -1.0
    d[f"DEF-thigh.{side}"] = q(X, -thigh) @ q(Y, -sign * out)
    d[f"DEF-shin.{side}"] = q(X, knee)
    d[f"DEF-foot.{side}"] = q(X, foot)


def trunk(d, lean, twist=0.0, bank=0.0, look=0.0, chin=0.0):
    """The spine bent forward by `lean` over its four bones, banked into a
    turn and twisted; the neck takes the lean back so the eyes stay level,
    and the head turns by `look`."""
    d["DEF-spine"] = q(X, 0.30 * lean) @ q(Y, -0.6 * bank) @ q(Z, 0.3 * twist)
    d["DEF-spine.001"] = q(X, 0.25 * lean) @ q(Z, -0.25 * twist)
    d["DEF-spine.002"] = q(X, 0.25 * lean) @ q(Y, -0.4 * bank) @ q(Z, -0.4 * twist)
    d["DEF-spine.003"] = q(X, 0.20 * lean) @ q(Z, -0.35 * twist)
    back = 0.1 * twist  # what's left of the chest's turn to undo
    for bone, share in (("DEF-spine.004", 0.4), ("DEF-spine.005", 0.35), ("DEF-spine.006", 0.25)):
        d[bone] = q(X, (-0.8 * lean + chin) * share) @ q(Z, (look - back) * share)


def stride(d, phase, amp, knee=0.0):
    """The legs and arms of a running stride at `phase`, `amp` its size
    (0 stands), knees bent a further `knee`."""
    s = math.sin(phase)
    for side, sign in (("L", 1.0), ("R", -1.0)):
        swing = s * sign  # +1: this leg forward
        recovery = max(0.0, math.sin(phase * sign + 0.9 * sign))
        leg(d, side, (0.65 * swing + 0.15) * amp + 0.6 * knee, (0.25 + 1.25 * recovery) * amp + 0.05 + knee, (-0.25 * max(0.0, -swing) + 0.15 * max(0.0, swing)) * amp)
        arm(d, side, 1.25, 0.47 * swing * amp, 0.25 + (0.75 + 0.2 * max(0.0, -swing)) * amp)


# ── The moves: (deltas, height in the air) at `t` ∈ [0, 1] through the move ──
def run_move(m):
    d = {}
    trunk(d, 0.14 * m.amp + 0.02, 0.30 * math.sin(m.phase) * m.amp, m.bank, m.look)
    stride(d, m.phase, m.amp)
    return d, 0.0


def dribble_move(m):
    d = {}
    trunk(d, 0.30 + 0.10 * m.amp, 0.15 * math.sin(m.phase) * m.amp, m.bank, m.look)
    stride(d, m.phase, 0.8 * m.amp, knee=0.30)
    arm(d, "R", 1.05, -0.45, 0.25 + 0.75 * m.bounce, curl=0.15, wrist=-0.3 + 0.5 * m.bounce)
    arm(d, "L", 1.0, -0.75, 1.3, curl=0.35)  # the off arm out front, guarding the ball
    return d, 0.0


def pass_move(m):
    """A chest pass: the ball drawn in, then both arms punched out, thumbs
    turning down; the release at 0.4."""
    d = {}
    out = ramp(m.t, 0.25, 0.42)
    trunk(d, 0.15 + 0.25 * out, 0.0, 0.0, m.look)
    stride(d, m.phase, 0.5 * m.amp, knee=0.15)
    for side in ("L", "R"):
        arm(d, side, mix(1.40, 1.25, out), mix(-0.95, -1.45, out), mix(2.0, 0.2, out), curl=mix(0.3, 0.1, out), wrist=mix(0.0, 0.5, out))
    return d, 0.0


def catch_move(m):
    """Hands out to meet the ball, then drawn in to the chest."""
    d = {}
    pull = ramp(m.t, 0.3, 0.6)
    trunk(d, 0.15, 0.0, 0.0, m.look)
    stride(d, m.phase, 0.5 * m.amp, knee=0.2)
    for side in ("L", "R"):
        arm(d, side, mix(1.35, 1.42, pull), mix(-1.40, -0.75, pull), mix(0.35, 1.65, pull), curl=mix(0.1, 0.35, pull))
    return d, 0.0


def jump_shot_move(m):
    """Dip, rise with the ball to the set point above the forehead, leave
    the floor, release at 0.55 at the top of the jump with a flick of the
    wrist, follow through and land."""
    d = {}
    dip = ramp(m.t, 0.0, 0.25) * (1 - ramp(m.t, 0.25, 0.38))
    land = ramp(m.t, 0.78, 0.86) * (1 - ramp(m.t, 0.88, 1.0))
    bend = max(dip, 0.8 * land)
    air = 0.42 * max(0.0, math.sin(math.pi * (m.t - 0.36) / 0.42)) if 0.36 < m.t < 0.78 else 0.0
    trunk(d, 0.15 + 0.35 * bend, 0.0, 0.0, m.look, chin=-0.1)
    for side in ("L", "R"):
        leg(d, side, 0.65 * bend + 0.05, 1.2 * bend + 0.08, -0.35 * (air > 0))
    rise = ramp(m.t, 0.1, 0.4)
    shoot = ramp(m.t, 0.5, 0.58)
    arm(d, "R", mix(1.35, 1.10, rise), mix(-0.9, -2.0, rise) - 0.65 * shoot, mix(2.0, 2.3, rise) - 2.1 * shoot, curl=0.2, wrist=mix(-0.6, 0.9, shoot))
    arm(d, "L", mix(1.35, 1.0, rise), mix(-0.9, -1.9, rise), mix(2.0, 1.5, rise), curl=0.15)
    return d, air


def layup_move(m):
    """The gather and the two steps (the stride carries them), then off the
    left foot with the right knee driving up, the ball carried high in the
    right hand and laid in at the top (0.68)."""
    d = {}
    jump = ramp(m.t, 0.48, 0.56)
    land = ramp(m.t, 0.88, 1.0)
    air = 0.55 * max(0.0, math.sin(math.pi * (m.t - 0.52) / 0.38)) if 0.52 < m.t < 0.9 else 0.0
    trunk(d, 0.25 - 0.15 * jump, 0.0, m.bank, m.look, chin=-0.15 * jump)
    stride(d, m.phase, (1 - jump) * m.amp, knee=0.15)
    if jump > 0:
        up = {}
        leg(up, "R", 1.45, 1.6, 0.2)  # the drive knee
        leg(up, "L", 0.05, 0.1, -0.5)  # the take-off leg, toes down
        for bone, rot in up.items():
            d[bone] = d.get(bone, Quaternion()).slerp(rot, jump * (1 - land))
    carry = ramp(m.t, 0.45, 0.68)
    arm(d, "R", mix(1.40, 1.10, carry), mix(-0.95, -2.85, carry), mix(2.0, 0.15, carry), curl=0.15, wrist=0.5 * ramp(m.t, 0.66, 0.72))
    arm(d, "L", mix(1.40, 1.15, carry), mix(-0.95, -1.6, carry), mix(2.0, 0.8, carry), curl=0.2)
    return d, air


def defend_move(m):
    """The stance: low, feet wide, back flat, one hand up at the ball and
    one low at the passing lane; small shuffling steps."""
    d = {}
    trunk(d, 0.55, 0.0, 0.0, m.look, chin=-0.15)
    s = math.sin(m.phase)
    for side, sign in (("L", 1.0), ("R", -1.0)):
        leg(d, side, 0.70 + 0.18 * s * sign * m.amp, 1.05 + 0.25 * max(0.0, s * sign) * m.amp, 0.15, out=0.28)
    arm(d, "L", 0.95, -2.2, 0.3, curl=0.1)
    arm(d, "R", 0.55, -0.35, 0.55, curl=0.1)
    return d, 0.0


def contest_move(m):
    """A straight-up jump, both arms high and straight: verticality."""
    d = {}
    dip = ramp(m.t, 0.0, 0.28) * (1 - ramp(m.t, 0.28, 0.38))
    land = ramp(m.t, 0.8, 0.88) * (1 - ramp(m.t, 0.9, 1.0))
    bend = max(dip, 0.8 * land)
    air = 0.38 * max(0.0, math.sin(math.pi * (m.t - 0.38) / 0.42)) if 0.38 < m.t < 0.8 else 0.0
    trunk(d, 0.2 + 0.3 * bend, 0.0, 0.0, m.look, chin=-0.2)
    for side in ("L", "R"):
        leg(d, side, 0.6 * bend + 0.05, 1.1 * bend + 0.08, -0.35 * (air > 0))
        up = ramp(m.t, 0.15, 0.4) * (1 - ramp(m.t, 0.85, 1.0))
        arm(d, side, mix(1.0, 1.12, up), mix(-0.6, -2.85, up), mix(0.8, 0.1, up), curl=0.05)
    return d, air


def pick_up_move(m):
    """Bend at hips and knees, reach down to the ball, take it at 0.45 and
    rise with it at the chest."""
    d = {}
    down = ramp(m.t, 0.0, 0.4) * (1 - ramp(m.t, 0.5, 0.85))
    trunk(d, 0.2 + 0.9 * down, 0.0, 0.0, m.look * (1 - down), chin=0.3 * down)
    stride(d, m.phase, 0.4 * m.amp * (1 - down), knee=0.75 * down)
    hold = ramp(m.t, 0.45, 0.8)
    for side in ("L", "R"):
        arm(d, side, 1.32, mix(-0.55, -0.95, hold), mix(0.25, 2.0, hold), curl=mix(0.1, 0.35, hold))
    return d, 0.0


def celebrate_move(m):
    """Two hops with both fists up in a V."""
    d = {}
    hop = abs(math.sin(2 * math.pi * min(m.t * 1.6, 1.0)))
    trunk(d, -0.05, 0.0, 0.0, m.look, chin=-0.25)
    stride(d, m.phase, 0.3 * m.amp, knee=0.25 * (1 - hop))
    up = ramp(m.t, 0.0, 0.2)
    for side in ("L", "R"):
        arm(d, side, mix(1.25, -0.75, up), 0.0, mix(0.3, 0.6 + 0.4 * hop, up), curl=0.95)
    return d, 0.14 * hop * (1 - ramp(m.t, 0.6, 0.75))


MOVES = {
    "Run": run_move,
    "Dribble": dribble_move,
    "Pass": pass_move,
    "Catch": catch_move,
    "Jump Shot": jump_shot_move,
    "Layup": layup_move,
    "Defend": defend_move,
    "Contest": contest_move,
    "Pick Up": pick_up_move,
    "Celebrate": celebrate_move,
}
RELEASE = {"Pass": 0.4, "Jump Shot": 0.55, "Layup": 0.68}  # when the ball leaves the hands
TAKE = {"Catch": 0.36, "Pick Up": 0.45}  # when it lands in them


# ── Solving a pose ──
REST_BONES = sorted(REST.bones, key=lambda b: len(b.parent_recursive))  # parents first
REST_M = {b.name: (b.matrix_local.copy(), b.matrix_local.to_quaternion().to_matrix().to_4x4()) for b in REST_BONES}
REL_INV = {b.name: ((b.parent.matrix_local.inverted() @ b.matrix_local) if b.parent else b.matrix_local).inverted() for b in REST_BONES}
LENGTH = {b.name: b.length for b in REST_BONES}
# Where each foot meets the floor at rest: the toe's tip, the ball of the
# foot, the heel under the ankle.
CONTACT = [(f"DEF-toe.{s}", True, REST.bones[f"DEF-toe.{s}"].tail_local.z) for s in "LR"]
CONTACT += [(f"DEF-toe.{s}", False, REST.bones[f"DEF-toe.{s}"].head_local.z) for s in "LR"]
CONTACT += [(f"DEF-foot.{s}", False, REST.bones[f"DEF-foot.{s}"].head_local.z) for s in "LR"]


SOLE = 0.022  # the sneakers' sole, under the foot's lowest point


def solve(deltas, air):
    """Each bone's basis rotation (and the root's location) for `deltas`,
    the body lifted so the lowest foot point stands on the floor, then
    `air` above it; and the posed armature-space matrices."""
    absolute, posed, basis = {}, {}, {}
    for b in REST_BONES:
        n = b.name
        p = b.parent
        absolute[n] = (absolute[p.name] @ deltas[n]) if (p and n in deltas) else (absolute[p.name] if p else deltas.get(n, Quaternion()))
        rest, rest_rot = REST_M[n]
        head = (posed[p.name] @ (REST_M[p.name][0].inverted() @ rest.translation)) if p else rest.translation.copy()
        target = Matrix.Translation(head) @ absolute[n].to_matrix().to_4x4() @ rest_rot
        posed[n] = target
        if n in deltas or p is None:
            basis[n] = REL_INV[n] @ (posed[p.name].inverted() if p else Matrix.Identity(4)) @ target
    lowest = min(((posed[n] @ Vector((0, LENGTH[n], 0))) if tip else posed[n].translation).z - rest_z for n, tip, rest_z in CONTACT)
    lift = air - lowest + SOLE
    root = REST_BONES[0].name
    shift = Matrix.Translation((0.0, 0.0, lift))
    basis[root] = REL_INV[root] @ shift @ posed[root]
    for n in posed:
        posed[n] = shift @ posed[n]
    return basis, posed


# ── The choreography ──
# A player's plan: where they go — along waypoints (`None`: from where they
# are) or marking a player toward a basket — and the moves they make; an
# "auto" move is Run or Defend by their pace.
PLANS = {
    "Ayanda": {
        "go": [("path", 1, 137, [(1, -0.5, 2.5), (30, -3.6, 2.9), (51, -5.7, 2.7), (65, -6.4, 2.4), (100, -9.4, 3.6), (137, -9.0, 4.2)]), ("mark", 138, FRAMES, "Dumi", 1)],
        "moves": [("Dribble", 1, 51), ("Pass", 52, 65), ("Run", 66, 137), ("auto", 138, FRAMES)],
    },
    "Buhle": {
        "go": [("path", 1, 160, [(1, -5.0, -3.2), (30, -6.8, -2.0), (55, -8.9, -4.0), (67, -8.4, -3.5), (74, -8.3, -3.4), (98, -9.4, -2.3), (130, -9.5, -2.2), (160, -9.0, -2.6)]), ("mark", 161, FRAMES, "Lerato", 1)],
        "moves": [("Run", 1, 62), ("Catch", 63, 74), ("Dribble", 75, 98), ("Jump Shot", 99, 130), ("Celebrate", 131, 160), ("auto", 161, 289), ("Contest", 290, 315), ("auto", 316, FRAMES)],
    },
    "Chuma": {
        "go": [("path", 1, 140, [(1, -2.5, -1.0), (45, -8.0, 4.5), (75, -11.0, 5.6), (140, -11.2, 5.6)]), ("mark", 141, FRAMES, "Sipho", 1)],
        "moves": [("Run", 1, 140), ("auto", 141, FRAMES)],
    },
    "Dumi": {
        "go": [("mark", 1, 137, "Ayanda", -1), ("path", 138, FRAMES, [(138, None), (156, -11.3, 0.9), (170, -11.1, 1.0), (200, -4.5, -1.2), (235, 2.6, -1.6), (249, 3.4, -1.8), (300, 7.0, -3.0), (FRAMES, 7.6, -3.2)])],
        "moves": [("auto", 1, 137), ("Run", 138, 156), ("Pick Up", 157, 170), ("Dribble", 171, 235), ("Pass", 236, 249), ("Run", 250, FRAMES)],
    },
    "Lerato": {
        "go": [("mark", 1, 131, "Buhle", -1), ("path", 132, FRAMES, [(132, None), (180, -2.0, 3.2), (232, 5.4, 2.7), (250, 6.2, 2.4), (277, 9.0, 1.4), (293, 10.8, 0.7), (311, 11.7, 0.3), (335, 10.6, 1.4), (FRAMES, 9.8, 2.0)])],
        "moves": [("auto", 1, 102), ("Contest", 103, 131), ("Run", 132, 246), ("Catch", 247, 258), ("Dribble", 259, 277), ("Layup", 278, 311), ("Celebrate", 312, FRAMES)],
    },
    "Sipho": {
        "go": [("mark", 1, 137, "Chuma", -1), ("path", 138, FRAMES, [(138, None), (200, 0.0, -4.5), (240, 6.0, -4.6), (FRAMES, 6.8, -4.2)])],
        "moves": [("auto", 1, 137), ("Run", 138, FRAMES)],
    },
}
# Who has the ball, frame by frame, and how: (kind, first, last, who).
CARRY = [
    ("dribble", 1, 51, "Ayanda"),
    ("hold", 52, 57, "Ayanda"),
    ("hold", 67, 74, "Buhle"),
    ("dribble", 75, 98, "Buhle"),
    ("hold", 99, 116, "Buhle"),
    ("hold", 163, 170, "Dumi"),
    ("dribble", 171, 235, "Dumi"),
    ("hold", 236, 241, "Dumi"),
    ("hold", 251, 258, "Lerato"),
    ("dribble", 259, 277, "Lerato"),
    ("hold", 278, 300, "Lerato"),
]
SCORED = {"Home": 140, "Away": 311}  # the frames the ball drops through the rim


def hermite(points, f):
    """Catmull-Rom through timed (frame, x, y) points; held past the ends."""
    if f <= points[0][0]:
        return Vector((points[0][1], points[0][2], 0.0))
    if f >= points[-1][0]:
        return Vector((points[-1][1], points[-1][2], 0.0))
    k = max(i for i, p in enumerate(points) if p[0] <= f)
    p0, p1 = points[k], points[k + 1]
    P = [Vector((p[1], p[2], 0.0)) for p in points]

    def tangent(i):
        if i == 0 or i == len(points) - 1:
            return Vector((0.0, 0.0, 0.0))
        return (P[i + 1] - P[i - 1]) / (points[i + 1][0] - points[i - 1][0])

    h = p1[0] - p0[0]
    s = (f - p0[0]) / h
    h00, h10, h01, h11 = 2 * s**3 - 3 * s**2 + 1, s**3 - 2 * s**2 + s, -2 * s**3 + 3 * s**2, s**3 - s**2
    return h00 * P[k] + h10 * h * tangent(k) + h01 * P[k + 1] + h11 * h * tangent(k + 1)


def ball_holder(f):
    for kind, a, b, who in CARRY:
        if a <= f <= b:
            return who
    return None


# Where everyone is, frame by frame: those on a path first, then the
# markers, who keep goal-side of their player — 1.5 m off them toward the
# basket, never nearer than an arm's length — lagging behind as a defender reacts, never faster than a
# sprint.
SPRINT = 7.5 / FPS
pos = {name: {} for name in PLANS}
paths = {}
for f in range(1, FRAMES + 1):
    for pass_ in ("path", "mark"):
        for name, plan in PLANS.items():
            go = next(g for g in plan["go"] if g[1] <= f <= g[2])
            if go[0] != pass_:
                continue
            prev = pos[name].get(f - 1)
            if go[0] == "path":
                key = (name, go[1])
                if key not in paths:
                    pts = [(p[0], *(p[1:] if p[1] is not None else (prev.x, prev.y))) for p in go[3]]
                    paths[key] = pts
                target = hermite(paths[key], f)
                if prev is None or f > go[1] + 2:
                    pos[name][f] = target
                    continue
                gain = 1.0
            else:
                man = pos[go[3]][f]
                hoop = HOOP[go[4]].to_2d().to_3d()
                target = man + (hoop - man).normalized() * 1.5
                gain = 0.3
            if prev is None:
                pos[name][f] = target
                continue
            step = (target - prev) * gain
            if step.length > SPRINT:
                step = step.normalized() * SPRINT
            at = prev + step
            if go[0] == "mark":
                # An arm's length off their player, never through them.
                gap = at - man
                if gap.length < 1.0:
                    at = man + (gap.normalized() if gap.length > 1e-4 else (hoop - man).normalized()) * 1.0
            pos[name][f] = at


def ball_guess(f):
    """Roughly where the ball is, for whom to face: its holder, else along
    its way between the last holder and the next."""
    who = ball_holder(f)
    if who:
        return pos[who][f]
    before = [c for c in CARRY if c[2] < f]
    after = [c for c in CARRY if c[1] > f]
    if not before or not after:
        return HOOP[-1 if f < 200 else 1].to_2d().to_3d()
    a, b = before[-1], after[0]
    w = (f - a[2]) / (b[1] - a[2])
    return pos[a[3]][a[2]].lerp(pos[b[3]][b[1]], w)


def heading_to(v):
    return math.atan2(v.x, -v.y)  # turn −Y onto v


def unwrap(angle, near):
    while angle - near > math.pi:
        angle -= 2 * math.pi
    while angle - near < -math.pi:
        angle += 2 * math.pi
    return angle


class Moment:
    """One frame of one player, as the moves read it."""


def resolve_moves(name):
    """The plan's moves with each "auto" span split into Run and Defend by
    pace (ten frames at least to each), as (move, first, last)."""
    out = []
    for move, a, b in PLANS[name]["moves"]:
        if move != "auto":
            out.append((move, a, b))
            continue
        frames = list(range(a, b + 1))
        fast = [(pos[name][f] - pos[name][max(f - 1, 1)]).length * FPS > 3.2 for f in frames]
        runs, cur, start = [], fast[0], a
        for f, v in zip(frames, fast):
            if v != cur and f - start >= 10:
                runs.append(("Run" if cur else "Defend", start, f - 1))
                cur, start = v, f
        runs.append(("Run" if cur else "Defend", start, b))
        out.extend(runs)
    merged = []  # one strip for a move that carries on
    for move, a, b in out:
        if merged and merged[-1][0] == move:
            merged[-1] = (move, merged[-1][1], b)
        else:
            merged.append((move, a, b))
    return merged


BLEND = 5  # frames a move takes to give way to the next
BONES = set()
frames_of = {}  # name → per frame: (location, heading, basis by bone, posed)
for name in PLANS:
    moves = resolve_moves(name)
    PLANS[name]["strips"] = moves
    move_at = {f: (m, a, b) for m, a, b in moves for f in range(a, b + 1)}
    heading, phase, dphase = None, 0.0, 0.0
    rows = []
    turns = []
    for f in range(1, FRAMES + 1):
        p = pos[name][f]
        v = (pos[name][min(f + 1, FRAMES)] - pos[name][max(f - 1, 1)]) * (FPS / 2)
        speed = v.to_2d().length
        move, a, b = move_at[f]
        near = ball_guess(f)
        # Facing: where they run when running, else the ball — the shooter
        # the basket, the celebrating player the crowd.
        if move in ("Run", "Dribble", "Layup") and speed > 1.5:
            want = heading_to(v)
        elif move == "Jump Shot" or (move == "Pick Up" and f > (a + b) / 2):
            want = heading_to(HOOP[-1 if name in ("Ayanda", "Buhle", "Chuma") else 1] - p)
        elif move == "Celebrate":
            want = 0.0
        else:
            to_ball = near - p
            want = heading_to(to_ball) if to_ball.length > 0.3 else (heading if heading is not None else 0.0)
        if heading is None:
            heading = want
        want = unwrap(want, heading)
        turn = max(-0.3, min(0.3, (want - heading) * 0.3))
        heading += turn
        turns.append(turn)
        # The stride's phase follows the ground covered: a cycle (two steps)
        # a stride length, longer the faster they go, so the feet do not skate.
        stride_m = max(1.1, min(2.6, 0.9 + 0.3 * speed))
        phase += 2 * math.pi * speed / FPS / stride_m
        cadence = max(1.8, 2 * speed / stride_m)  # a bounce a step, never slower than 1.8 a second
        dphase += math.pi * cadence / FPS
        rows.append((f, p.copy(), heading, speed, move, a, b, phase, dphase, near))
    keyed = []
    for i, (f, p, h, speed, move, a, b, phase, dphase, near) in enumerate(rows):
        window = turns[max(0, i - 4): i + 5]
        rate = sum(window) / len(window) * FPS
        m = Moment()
        m.amp = min(1.0, speed / 5.5) ** 0.7
        m.phase = phase
        m.bounce = abs(math.sin(dphase))
        m.bank = max(-0.3, min(0.3, 0.07 * rate * speed))
        look_at = near - p
        m.look = max(-0.9, min(0.9, unwrap(heading_to(look_at), h) - h)) if look_at.length > 0.3 else 0.0
        m.t = (f - a) / max(1, b - a)
        deltas, air = MOVES[move](m)
        # The move before gives way over the first frames of this one.
        if f - a < BLEND and a > 1:
            pm, pa, pb = move_at[a - 1]
            prev = Moment()
            prev.__dict__.update(m.__dict__)
            prev.t = 1.0
            pd, pair = MOVES[pm](prev)
            w = smooth((f - a + 1) / (BLEND + 1))
            for bone in set(pd) | set(deltas):
                deltas[bone] = pd.get(bone, Quaternion()).slerp(deltas.get(bone, Quaternion()), w)
            air = mix(pair, air, w)
        basis, posed = solve(deltas, air)
        BONES.update(n for n in basis if n != REST_BONES[0].name)
        keyed.append((p, h, basis, posed))
    frames_of[name] = keyed
    print("played", name, [m for m, _, _ in moves], flush=True)


# ── The players' actions, as NLA strips ──
def world_of(name, f):
    p, h, _, _ = frames_of[name][f - 1]
    return Matrix.Translation(p) @ Matrix.Rotation(h, 4, "Z")


def hand(name, f, side):
    """The palm's end of a hand, in the world."""
    _, _, _, posed = frames_of[name][f - 1]
    bone = f"DEF-hand.{side}"
    return world_of(name, f) @ (posed[bone] @ Vector((0.0, LENGTH[bone], 0.0)))


def write_curve(action, ob, path, index, group, frames, values):
    fc = action.fcurve_ensure_for_datablock(ob, path, index=index, group_name=group)
    fc.keyframe_points.add(len(frames))
    fc.keyframe_points.foreach_set("co", [c for fv in zip(frames, values) for c in fv])
    fc.update()
    return fc


def thinned(frames, values, tol=1.5e-3):
    """The keys a curve needs: the first and last, and between them only
    where a straight line from the last kept key misses a dropped one by
    more than `tol` — so a held value is one key each end, a steady
    turn a key each end of it."""
    keep = [0]
    i = 0
    while i < len(frames) - 1:
        j = i + 1
        while j + 1 < len(frames):
            a, b = keep[-1], j + 1
            line = lambda k: values[a] + (values[b] - values[a]) * (frames[k] - frames[a]) / (frames[b] - frames[a])
            if all(abs(values[k] - line(k)) <= tol for k in range(a + 1, b)):
                j += 1
            else:
                break
        keep.append(j)
        i = j
    return [frames[k] for k in keep], [values[k] for k in keep]


def key_span(action, ob, channels, a, b, step=2):
    """Key `channels` — (path, index, group, value of frame) — over frames
    `a`…`b`, sampled every `step` frames and the last and then thinned to
    the keys the curve needs (`thinned`)."""
    frames = sorted(set(range(a, b + 1, step)) | {b})
    for path, index, group, value in channels:
        values = [value(f) for f in frames]
        if max(values) - min(values) < 1e-5:
            write_curve(action, ob, path, index, group, [a], [values[0]])
        else:
            write_curve(action, ob, path, index, group, *thinned(frames, values))


def lay_strips(ob, track_name, spans, channels):
    """Each (name, first, last) span as its own action keyed from
    `channels`, laid back to back on a new track of `ob`'s NLA."""
    ob.animation_data_create()
    track = ob.animation_data.nla_tracks.new()
    track.name = track_name
    for label, a, b in spans:
        action = bpy.data.actions.new(label)
        ob.animation_data.action = action
        key_span(action, ob, channels, a, b)
        ob.animation_data.action = None
        strip = track.strips.new(label, a, action)
        strip.action_slot = action.slots[0]
        strip.extrapolation = "HOLD" if a == 1 else "HOLD_FORWARD"
    return track


HOME = {"name": "Home", "jersey": PAL["scarlet"], "trim": PAL["poppy"], "shorts": PAL["scarlet"], "numbers": PAL["white"]}
AWAY = {"name": "Away", "jersey": PAL["bluebell"], "trim": PAL["sage"], "shorts": PAL["bluebell"], "numbers": PAL["gold"]}
kits = {"Home": team_kit(HOME), "Away": team_kit(AWAY)}
ROSTER = [("Ayanda", HOME, 7), ("Buhle", HOME, 11), ("Chuma", HOME, 23), ("Dumi", AWAY, 3), ("Lerato", AWAY, 14), ("Sipho", AWAY, 30)]
ORDERED = sorted(BONES, key=lambda n: len(REST.bones[n].parent_recursive))
rigs = {}
for name, team, number in ROSTER:
    rig = rigs[name] = player(name, team, number, kits[team["name"]])
    for b in rig.pose.bones:
        b.rotation_mode = "QUATERNION"
    rows = frames_of[name]
    # A quaternion and its negation are one turn; keep each bone's on one
    # side so the curves between keys take the short way.
    quats = {n: [] for n in ORDERED}
    for _, _, basis, _ in rows:
        for n in ORDERED:
            qn = basis[n].to_quaternion() if n in basis else Quaternion()
            if quats[n] and quats[n][-1].dot(qn) < 0:
                qn.negate()
            quats[n].append(qn)
    root = REST_BONES[0].name
    lifts = [basis[root].translation.copy() for _, _, basis, _ in rows]
    headings = [h for _, h, _, _ in rows]
    channels = [("location", i, "Object Transforms", (lambda f, i=i: rows[f - 1][0][i])) for i in range(3)]
    channels.append(("rotation_euler", 2, "Object Transforms", lambda f: headings[f - 1]))
    channels += [(f'pose.bones["{root}"].location', i, root, (lambda f, i=i: lifts[f - 1][i])) for i in range(3)]
    finger = lambda n: n.startswith(("DEF-f_", "DEF-thumb"))
    channel = lambda n, i: (f'pose.bones["{n}"].rotation_quaternion', i, n, (lambda f, n=n, i=i: quats[n][f - 1][i]))
    channels += [channel(n, i) for n in ORDERED if not finger(n) for i in range(4)]
    spans = [(f"{name} {move}", a, b) for move, a, b in PLANS[name]["strips"]]
    # Names are unique per action: a second Run is "Run.001" in Blender's
    # way, so number repeats.
    seen = {}
    for k, (label, a, b) in enumerate(spans):
        seen[label] = seen.get(label, 0) + 1
        if seen[label] > 1:
            spans[k] = (f"{label} {seen[label]}", a, b)
    lay_strips(rig, "Moves", spans, channels)
    # The fingers on a track above: one strip for the whole game, its keys
    # where a hand opens or closes — over the moves, which leave them be.
    lay_strips(rig, "Hands", [(f"{name} Hands", 1, FRAMES)], [channel(n, i) for n in ORDERED if finger(n) for i in range(4)])
    print("laid", name, len(spans), "strips", flush=True)


# ── The ball ──
def flight(p0, p1, a, b):
    """A throw from `p0` at frame `a` to `p1` at `b` under gravity."""
    T = (b - a) / FPS
    v0 = (p1 - p0) / T + Vector((0.0, 0.0, 4.905 * T))
    return lambda f: p0 + v0 * ((f - a) / FPS) + Vector((0.0, 0.0, -4.905 * ((f - a) / FPS) ** 2))


def in_hands(who, f):
    return (hand(who, f, "L") + hand(who, f, "R")) / 2 + Vector((0.0, 0.0, -0.02))


def dribbled(who, f):
    """Under the right palm at the top of a bounce, on the floor at the
    bottom: the same clock as the hand pumping it."""
    top = hand(who, f, "R") + Vector((0.0, 0.0, -0.13))
    i = f - 1
    bounce = abs(math.sin(dribble_phase[who][i]))
    return Vector((top.x, top.y, BALL_R + (top.z - BALL_R) * bounce))


dribble_phase = {}
for name in PLANS:
    # The same clock the dribbling move used, rebuilt from the paces.
    ph, out = 0.0, []
    for f in range(1, FRAMES + 1):
        v = (pos[name][min(f + 1, FRAMES)] - pos[name][max(f - 1, 1)]) * (FPS / 2)
        speed = v.to_2d().length
        stride_m = max(1.1, min(2.6, 0.9 + 0.3 * speed))
        ph += math.pi * max(1.8, 2 * speed / stride_m) / FPS
        out.append(ph)
    dribble_phase[name] = out

rim = {s: HOOP[s] + Vector((0.0, 0.0, 0.03)) for s in (-1, 1)}
net = {s: HOOP[s] + Vector((0.0, 0.0, -0.6)) for s in (-1, 1)}
shot = flight(in_hands("Buhle", 116) + Vector((0, 0, 0.1)), rim[-1], 116, 138)
to_buhle = flight(in_hands("Ayanda", 57), in_hands("Buhle", 67), 57, 67)
to_lerato = flight(in_hands("Dumi", 241), in_hands("Lerato", 251), 241, 251)
layup = flight(hand("Lerato", 300, "R") + Vector((0, 0, 0.1)), rim[1], 300, 309)
west_floor = Vector((-12.0, 0.45, BALL_R))
fall_w = flight(net[-1], west_floor, 143, 148)
to_dumi = flight(west_floor, in_hands("Dumi", 163), 148, 163)
east_floor = Vector((11.9, 0.35, BALL_R))
fall_e = flight(net[1], east_floor, 314, 320)
bounces = [(320, 334, east_floor, Vector((11.1, 0.8, BALL_R))), (334, 344, Vector((11.1, 0.8, BALL_R)), Vector((10.6, 1.1, BALL_R))), (344, 351, Vector((10.6, 1.1, BALL_R)), Vector((10.3, 1.3, BALL_R)))]


def ball_at(f):
    who = ball_holder(f)
    if who:
        kind = next(c[0] for c in CARRY if c[1] <= f <= c[2])
        return dribbled(who, f) if kind == "dribble" else in_hands(who, f)
    if 57 < f < 67:
        return to_buhle(f)
    if 116 < f <= 138:
        return shot(f)
    if 138 < f <= 143:
        return rim[-1].lerp(net[-1], smooth((f - 138) / 5))
    if 143 < f <= 148:
        return fall_w(f)
    if 148 < f < 163:
        return to_dumi(f)
    if 241 < f < 251:
        return to_lerato(f)
    if 300 < f <= 309:
        return layup(f)
    if 309 < f <= 314:
        return rim[1].lerp(net[1], smooth((f - 309) / 5))
    if 314 < f <= 320:
        return fall_e(f)
    for a, b, p0, p1 in bounces:
        if a < f <= b:
            return flight(p0, p1, a, b)(f)
    return Vector((10.3, 1.3, BALL_R))


ball_path = [ball_at(f) for f in range(1, FRAMES + 1)]
# It rolls with its travel along the floor and spins in the air.
spin = [Vector((0.0, 0.0, 0.0))]
for f in range(2, FRAMES + 1):
    step = ball_path[f - 1] - ball_path[f - 2]
    turn = Vector((-step.y, step.x, 0.0)) / BALL_R
    spin.append(spin[-1] + turn)
BALL_SPANS = [
    ("Ball Ayanda Dribble", 1, 51), ("Ball Pass to Buhle", 52, 74), ("Ball Buhle Dribble", 75, 98), ("Ball Jump Shot", 99, 148),
    ("Ball Rebound", 149, 170), ("Ball Dumi Dribble", 171, 235), ("Ball Pass to Lerato", 236, 258), ("Ball Lerato Dribble", 259, 277),
    ("Ball Layup", 278, 320), ("Ball Bounce Out", 321, FRAMES),
]
ball_channels = [("location", i, "Object Transforms", (lambda f, i=i: ball_path[f - 1][i])) for i in range(3)]
ball_channels += [("rotation_euler", i, "Object Transforms", (lambda f, i=i: spin[f - 1][i])) for i in range(2)]
lay_strips(ball, "Play", BALL_SPANS, ball_channels)

# Every throw glows, from the release to the catch or the net: a lit copy
# of the ball, its child, sits just inside it and steps just outside it for
# the flight. A switch of scale, keyed constant like the scoreboard's
# digits, so it plays wherever transforms do — no driven material.
THROWS = [(57, 67), (116, 143), (241, 251), (300, 314)]
glow_mat = material("Basketball Glow", (1.0, 0.55, 0.2), rough=0.4, image=BALL_LEATHER, emit=2.2)
glow_mesh = ball.data.copy()
glow_mesh.name = "Basketball Glow"
glow_mesh.materials.clear()
glow = mesh_object("Basketball Glow", glow_mesh, [glow_mat])
glow.parent = ball
glow.animation_data_create()
glow.animation_data.action = bpy.data.actions.new("Basketball Glow")
for f, lit in [(1, False)] + [(f, lit) for a, b in THROWS for f, lit in ((a, True), (b, False))]:
    glow.scale = (1.025,) * 3 if lit else (0.96,) * 3
    glow.keyframe_insert("scale", frame=f)
for layer in glow.animation_data.action.layers:
    for strip in layer.strips:
        for bag in strip.channelbags:
            for fc in bag.fcurves:
                for k in fc.keyframe_points:
                    k.interpolation = "CONSTANT"

# ── The scoreboard, hung over the north stand ─────────────────────────────
SB = Vector((0.0, 11.4, 4.3))
sb_ink = material("Scoreboard", PAL["ink"], rough=0.5)
sb_rim = material("Scoreboard Rim", PAL["poppy"], rough=0.4, emit=1.0)
box("Scoreboard", sb_ink, SB, (7.6, 0.5, 2.8))
box("Scoreboard Rim.T", sb_rim, SB + Vector((0, -0.26, 1.32)), (7.6, 0.04, 0.16))
box("Scoreboard Rim.B", sb_rim, SB + Vector((0, -0.26, -1.32)), (7.6, 0.04, 0.16))
FACE = (math.radians(90), 0.0, 0.0)  # text faces −Y, toward the court
labels = {"Home": (-1.9, PAL["scarlet"]), "Away": (1.9, PAL["bluebell"])}
face_y = SB.y - 0.27
for team, (x, color) in labels.items():
    mat = material(f"{team} Label", color, rough=0.4, emit=2.0)
    text_mesh(f"Scoreboard {team}", team.upper(), 0.5, (x, face_y, SB.z + 0.75), FACE, mat)
digit_mat = material("Score Digits", PAL["gold"], rough=0.3, emit=3.0)
text_mesh("Scoreboard LA28", "LA28", 0.28, (0.0, face_y, SB.z + 0.95), FACE, material("Scoreboard Mark", PAL["white"], rough=0.4, emit=1.5))
for team, (x, _) in labels.items():
    for points in (0, 2):
        digit = text_mesh(f"{team} Points {points}", str(points), 1.3, (x, face_y, SB.z - 0.35), FACE, digit_mat)
        # Shown before the basket ("0") or after it ("2"): the other sits
        # inside the board. Constant keys switch them on the frame.
        shown = (lambda f, p=points, t=team: (f < SCORED[t]) == (p == 0))
        digit.animation_data_create()
        digit.animation_data.action = bpy.data.actions.new(f"{team} Points {points}")
        for f in (1, SCORED[team]):
            digit.location.y = face_y if shown(f) else face_y + 0.6
            digit.keyframe_insert("location", index=1, frame=f)
        for layer in digit.animation_data.action.layers:
            for strip in layer.strips:
                for bag in strip.channelbags:
                    for fc in bag.fcurves:
                        for k in fc.keyframe_points:
                            k.interpolation = "CONSTANT"

# ── The crowd ─────────────────────────────────────────────────────────────
# Fans are one character, seated: eight versions of her — cheering or
# clapping, in four shirt colours of the palette — each a collection kept
# out of the scene, and an empty on a seat instancing one of them (a
# collection instance). The seats' places pick who sits where.
def fan_pose(kind):
    d = {}
    trunk(d, -0.12 if kind == "Cheer" else 0.12, chin=-0.15)
    for side in ("L", "R"):
        leg(d, side, 1.45, 1.5, 0.1, out=0.1)
        if kind == "Cheer":
            arm(d, side, -0.6, -0.25, 0.35, curl=0.95)
        else:
            arm(d, side, 1.25, -1.05, 1.75, curl=0.2)
    return d


def baked_fan(label, shirt, kind, tris=1600):
    """A fan as one plain mesh: her body, hair and a shirt and shorts, posed
    seated, decimated to about `tris` triangles, the seat at her origin."""
    basis, posed = solve(fan_pose(kind), 0.0)
    rig = link(bpy.data.objects.new("_fan", REST.copy()))
    bpy.context.view_layer.update()  # a new rig has no pose until evaluated
    root = REST_BONES[0].name
    for n, m in basis.items():
        pb = rig.pose.bones[n]
        pb.rotation_mode = "QUATERNION"
        pb.rotation_quaternion = m.to_quaternion()
        if n == root:
            pb.location = m.translation
    top = VEST.copy()
    top.materials.append(material(f"Fan Shirt {label}", shirt, rough=0.6))
    low = SHORTS.copy()
    low.materials.append(bpy.data.materials.get("Fan Shorts") or material("Fan Shorts", PAL["ink"], rough=0.6))
    meshes = [ob.data for role, ob in TEMPLATE.items() if role in ("Body", "Hair", "Ponytail", "Nose")] + [top, low]
    parts = []
    dg = bpy.context.evaluated_depsgraph_get()
    for mesh in meshes:
        part = link(bpy.data.objects.new("_part", mesh))
        part.modifiers.new("Armature", "ARMATURE").object = rig
        parts.append(part)
    dg = bpy.context.evaluated_depsgraph_get()
    baked = []
    for part in parts:
        flat = bpy.data.meshes.new_from_object(part.evaluated_get(dg))
        baked.append(link(bpy.data.objects.new("_baked", flat)))
    for part in parts:
        bpy.data.objects.remove(part)
    bpy.data.objects.remove(rig)
    for m in (top, low):
        bpy.data.meshes.remove(m)
    activate(baked[0])
    for b in baked[1:]:
        b.select_set(True)
    bpy.ops.object.join()
    fan = bpy.context.active_object
    count = sum(len(p.vertices) - 2 for p in fan.data.polygons)
    dec = fan.modifiers.new("Decimate", "DECIMATE")
    dec.ratio = min(1.0, tris / max(count, 1))
    dg = bpy.context.evaluated_depsgraph_get()
    mesh = bpy.data.meshes.new_from_object(fan.evaluated_get(dg))
    bpy.data.objects.remove(fan)
    # Her seat: between the thighs' heads, down to where she sits on it.
    hips = (posed["DEF-thigh.L"].translation + posed["DEF-thigh.R"].translation) / 2
    mesh.transform(Matrix.Translation(-Vector((0.0, hips.y + 0.05, hips.z - 0.11))))
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.name = f"Fan {label} {kind}"
    ob = bpy.data.objects.new(mesh.name, mesh)
    coll = bpy.data.collections.new(mesh.name)
    coll.objects.link(ob)
    return coll


FAN_SHIRTS = [("Scarlet", PAL["scarlet"]), ("Bluebell", PAL["bluebell"]), ("Poppy", PAL["poppy"]), ("Gold", PAL["gold"])]
FANS = [baked_fan(label, colour, kind) for label, colour in FAN_SHIRTS for kind in ("Cheer", "Clap")]
crowd = bpy.data.collections.new("Crowd")
scene.collection.children.link(crowd)
rng = np.random.default_rng(28)
seated = 0
for tier, at, heading in SEATS:
    if rng.random() > (0.22 if tier < 4 else 0.08):
        continue
    fan = bpy.data.objects.new(f"Fan.{seated:03d}", None)
    fan.instance_type = "COLLECTION"
    fan.instance_collection = FANS[int(rng.integers(len(FANS)))]
    fan.empty_display_size = 0.0
    fan.location = at + Vector((0.0, 0.0, SEAT_H))
    fan.rotation_euler = (0.0, 0.0, heading + float(rng.uniform(-0.2, 0.2)))
    crowd.objects.link(fan)
    seated += 1
print("crowd", seated, "fans,", len(SEATS), "seats", flush=True)

# ── Light, world, camera ──────────────────────────────────────────────────
for k, (x, y) in enumerate(((-7, -4), (7, -4), (-7, 4), (7, 4))):
    data = bpy.data.lights.new(f"Arena Light {k}", type="AREA")
    data.energy = 900.0
    data.size = 6.0
    data.color = lin((1.0, 0.96, 0.9))
    lamp = link(bpy.data.objects.new(f"Arena Light {k}", data))
    lamp.location = (x, y, 10.0)
sun_data = bpy.data.lights.new("Sun", type="SUN")
sun_data.energy = 1.5
sun = link(bpy.data.objects.new("Sun", sun_data))
sun.rotation_euler = Euler((math.radians(40), 0.0, math.radians(30)))

world = bpy.data.worlds.new("World")
scene.world = world
world.use_nodes = True
bg = world.node_tree.nodes["Background"]
bg.inputs["Color"].default_value = (*lin((0.16, 0.18, 0.28)), 1.0)
bg.inputs["Strength"].default_value = 1.0
world.color = lin((0.16, 0.18, 0.28))

cam_data = bpy.data.cameras.new("Camera")
cam_data.lens = 30.0
cam = link(bpy.data.objects.new("Camera", cam_data))
cam.location = (0.0, -21.0, 10.0)
cam.rotation_euler = Euler((math.radians(65), 0.0, 0.0))
scene.camera = cam
# A courtside camera low on the home half, for close looks at the players
# (`phiber -b … --camera Courtside`).
side_data = bpy.data.cameras.new("Courtside")
side_data.lens = 45.0
side = link(bpy.data.objects.new("Courtside", side_data))
side.location = (-4.0, -10.5, 1.7)
side.rotation_euler = (Vector((-6.5, 0.0, 1.0)) - side.location).to_track_quat("-Z", "Y").to_euler()

# Saved with everything animated selected (the first player active): the
# editors list every move the moment the file opens.
for obj in scene.objects:
    obj.select_set(obj.animation_data is not None)
bpy.context.view_layer.objects.active = rigs["Ayanda"]

# Solid ▸ Texture with shadows and cavity in every 3D view, the camera's
# angle in the main one.
for screen in bpy.data.screens:
    for area in screen.areas:
        for space in area.spaces:
            if space.type == "VIEW_3D":
                sh = space.shading
                sh.type = "SOLID"
                sh.light = "STUDIO"
                sh.color_type = "TEXTURE"
                sh.show_shadows = True
                sh.shadow_intensity = 0.45
                sh.show_cavity = True
                sh.cavity_type = "BOTH"
                # The arena is the floor: no grid or axes drawn across it.
                space.overlay.show_bones = False
                space.overlay.show_floor = False
                space.overlay.show_axis_x = False
                space.overlay.show_axis_y = False
                r3d = space.region_3d
                r3d.view_location = (0.0, 0.0, 0.8)
                r3d.view_rotation = Euler((math.radians(62), 0.0, 0.0)).to_quaternion()
                r3d.view_distance = 24.0

scene.frame_set(1)
# No `demo-court.blend1` beside it: `assets/blend` is embedded whole.
bpy.context.preferences.filepaths.save_version = 0
bpy.ops.wm.save_as_mainfile(filepath=os.path.abspath(OUT), compress=True, copy=True)
print("wrote", OUT, os.path.getsize(OUT), "bytes,", len(bpy.data.objects), "objects,", sum(len(o.data.vertices) for o in scene.objects if o.type == "MESH"), "vertices")
