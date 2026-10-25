"""The Phiber mark, cut in a triangle on each side.

On each side a cone starts at the middle of the inner line and opens outward
along the bracket's axis of symmetry. It makes three cuts: one through the
inner line at the cone's tip, and one through the outer line along each of the
cone's two edges. The cuts are drawn as clip paths, so each cut edge runs
along the cone rather than square to the line, and the pieces either side of
the cuts are mirror images (the cone is symmetric about the axis).

Previews render with resvg — the renderer GPUI uses — so they match the app.
"""
import io, math, os, colorsys
import resvg_py
from PIL import Image, ImageDraw, ImageFont

LINE = float(os.environ.get("LINE", "0.65"))
PAD = float(os.environ.get("PAD", "0.65"))
SUFFIX = os.environ.get("SUFFIX", "")
CONE = math.radians(float(os.environ.get("CONE", "35")))   # half-angle
CUT = float(os.environ.get("CUT", str(max(0.8, LINE))))     # width of each cut
CORE = 1.1
STEM = CORE + 2 * PAD
INSET = 2.325
R_CORNER = 1.4
W, STEM_X = 17.368 - 14.25, 12.0

def f(v): return f"{v:.3f}"

def rounded(pts, r):
    out = f"M{f(pts[0][0])} {f(pts[0][1])}"
    for i in range(1, len(pts) - 1):
        p0, p1, p2 = pts[i-1], pts[i], pts[i+1]
        def toward(a, b, dist):
            L = math.hypot(b[0]-a[0], b[1]-a[1]); return (a[0]+(b[0]-a[0])*dist/L, a[1]+(b[1]-a[1])*dist/L)
        a, b = toward(p1, p0, r), toward(p1, p2, r)
        out += f"L{f(a[0])} {f(a[1])}Q{f(p1[0])} {f(p1[1])} {f(b[0])} {f(b[1])}"
    return out + f"L{f(pts[-1][0])} {f(pts[-1][1])}"

def hex_line(xo, yt, yb, d, w, r, sx=None, inset=0.0):
    """Half hexagon, optionally offset `inset` inward and started at sx.
    Returns the path and the x of its vertical side (its middle)."""
    h = yb - yt
    slope, L = (h / 6) / w, math.hypot(w, h / 6)
    xv = xo + d * (w - inset)
    ox, oy = xo - d * inset * (h / 6) / L, yt + inset * w / L
    top = lambda x: oy + slope * d * (x - ox)
    bot = lambda x: (yb - (oy - yt)) - slope * d * (x - ox)
    x0 = xo if sx is None else sx
    return rounded([(x0, top(x0)), (xv, top(xv)), (xv, bot(xv)), (x0, bot(x0))], r), xv

def arc_line(xo, yt, yb, d, w, sx=None, inset=0.0):
    """Circular arc (or the same centre, `inset` smaller). Returns the path and
    the x of its middle (its outermost point)."""
    h, ym = yb - yt, (yt + yb) / 2
    R = (h * h / 4 + w * w) / (2 * w)
    cx = xo + d * (w - R)
    r = R - inset
    x0 = xo if sx is None else sx
    half = math.sqrt(max(r * r - (x0 - cx) ** 2, 0.0))
    sweep = 1 if d > 0 else 0
    return f"M{f(x0)} {f(ym - half)}A{f(r)} {f(r)} 0 0 {sweep} {f(x0)} {f(ym + half)}", cx + d * r

def strip(ax, ay, ux, uy, width, length=30.0):
    """A band `width` wide along direction (ux, uy) from (ax, ay) outward."""
    nx, ny = -uy * width / 2, ux * width / 2
    bx, by = ax + ux * length, ay + uy * length
    # Start a little behind the tip so the band fully crosses the line there.
    sx, sy = ax - ux * 1.0, ay - uy * 1.0
    return (f"M{f(sx+nx)} {f(sy+ny)}L{f(bx+nx)} {f(by+ny)}L{f(bx-nx)} {f(by-ny)}L{f(sx-nx)} {f(sy-ny)}Z")

UID = [0]
def clipped(path, width, bands):
    """`path` stroked, with every band cut out of it (even-odd clip)."""
    UID[0] += 1
    cid = f"cut{UID[0]}"
    clip = f'<clipPath id="{cid}" clipPathUnits="userSpaceOnUse"><path clip-rule="evenodd" d="M-20 -20H44V44H-20Z{"".join(bands)}"/></clipPath>'
    return clip + f'<path clip-path="url(#{cid})" fill="none" stroke="currentColor" stroke-width="{width}" stroke-linecap="round" stroke-linejoin="round" d="{path}"/>'

def bracket(kind, nested, xo, yt, yb, d):
    """[svg for the outer line, svg for the inner line?] of one side."""
    ym = (yt + yb) / 2
    outer, _ = hex_line(xo, yt, yb, d, W, R_CORNER) if kind == "hex" else arc_line(xo, yt, yb, d, W)
    sx = STEM_X + d * (CORE / 2 + LINE / 2)          # inner line runs under the stem
    inner, tip_x = (hex_line(xo, yt, yb, d, W, R_CORNER * 0.6, sx, INSET) if kind == "hex"
                    else arc_line(xo, yt, yb, d, W, sx, INSET))
    # The cone: tip at the middle of the inner line, edges at ±CONE about +d·x.
    edges = [strip(tip_x, ym, d * math.cos(CONE), s * math.sin(CONE), CUT) for s in (-1, 1)]
    out = [clipped(outer, LINE, edges)]
    if nested:
        out.append(clipped(inner, LINE, [strip(tip_x - d * 1.0, ym, d, 0.0, CUT, length=2.0)]))
    return out

def capsule(x, y0, y1, r):
    return (f"M{f(x - r)} {f(y0)}A{f(r)} {f(r)} 0 0 1 {f(x + r)} {f(y0)}"
            f"L{f(x + r)} {f(y1)}A{f(r)} {f(r)} 0 0 1 {f(x - r)} {f(y1)}Z")

def wrap(body, scale="1.25000"):
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">'
            f'<g transform="translate(12 12) scale({scale}) translate(-12 -12)"><g transform="rotate(30 12 12)">'
            + body + '</g></g></svg>')

def white(right, left, nested):
    body = "".join(bracket(right, nested, 14.25, 4.5, 15.3, +1) + bracket(left, nested, 9.75, 8.7, 19.5, -1))
    body += f'<path fill="currentColor" fill-rule="evenodd" d="{capsule(12, 4.5, 19.5, STEM/2)}{capsule(12, 4.9, 19.1, CORE/2)}"/>'
    return wrap(body)

def hsl(h, s, l):
    r, g, b = colorsys.hls_to_rgb(h/360, l/100, s/100); return "#%02X%02X%02X" % (round(r*255), round(g*255), round(b*255))
WARM, COOL = hsl(38, 92, 50), hsl(217, 91, 60)

def stroke(d, w):
    return f'<path fill="none" stroke="currentColor" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round" d="{d}"/>'

def layers(right, left, nested):
    """Bottom to top: cool (left bracket, right inner), warm (right bracket,
    left inner, stem), line (the stem's core) — colours alternate by level."""
    r = bracket(right, nested, 14.25, 4.5, 15.3, +1)
    l = bracket(left, nested, 9.75, 8.7, 19.5, -1)
    under = l[0] + "".join(r[1:])
    warm = r[0] + "".join(l[1:]) + stroke("M12 4.5v15", STEM)
    return under, warm, stroke("M12 4.9v14.2", CORE)

VARIANTS = {"phiber_arc": ("Arc left", ("hex", "semi", False)),
            "phiber_arc_nested": ("Arc left, nested", ("hex", "semi", True)),
            "phiber_arc_right_nested": ("Arc right, nested", ("semi", "hex", True))}

def png(svg, px):
    svg = svg.replace('width="24" height="24"', f'width="{px}" height="{px}"', 1)
    return Image.open(io.BytesIO(bytes(resvg_py.svg_to_bytes(svg_string=svg)))).convert("RGBA")

for name, (_, args) in VARIANTS.items():
    n = f"{name}_tri{SUFFIX}"
    open(f"{n}.svg", "w").write(white(*args))
    under, warm, line = layers(*args)
    open(f"{n}_cool.svg", "w").write(wrap(under, "1.48148"))
    open(f"{n}_warm.svg", "w").write(wrap(warm, "1.48148"))
    open(f"{n}_line.svg", "w").write(wrap(line, "1.48148"))
    open(f"{n}_color.svg", "w").write(wrap(f'<g color="{COOL}">{under}</g><g color="{WARM}">{warm}</g><g color="{COOL}">{line}</g>'))

font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 15)
cell, pad = 300, 24
sheet = Image.new("RGB", (pad + 3 * (cell + pad), 2 * 480 + 10), (30, 32, 36))
d = ImageDraw.Draw(sheet)
for row, colour in enumerate((False, True)):
    y0 = row * 480
    for i, (name, (label, _)) in enumerate(VARIANTS.items()):
        n = f"{name}_tri{SUFFIX}"
        mono = open(f"{n}.svg").read()
        big = open(f"{n}_color.svg").read() if colour else mono.replace("currentColor", "#E8ECF2")
        small = open(f"{n}_color.svg").read() if colour else mono.replace("currentColor", "#1F2328")
        x = pad + i * (cell + pad)
        d.rounded_rectangle([x, y0 + 20, x + cell, y0 + 20 + cell], 18, fill=(22, 24, 28))
        im = png(big, 240); sheet.paste(im, (x + 30, y0 + 50), im)
        d.rounded_rectangle([x, y0 + 340, x + cell, y0 + 430], 14, fill=(236, 238, 242))
        for j, px in enumerate((64, 32, 24, 16)):
            im = png(small, px); sheet.paste(im, (x + 20 + j * 72, y0 + 385 - px // 2), im)
        d.text((x, y0 + 436), label, fill=(220, 224, 230), font=font)
        d.text((x, y0 + 454), f"{n}{'_color' if colour else ''}.svg", fill=(140, 148, 160), font=font)
sheet.save(f"phiber_mark_tri{SUFFIX}.png")
print("ok", SUFFIX)
