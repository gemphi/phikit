"""The Phiber mark with a cone cut through each bracket.

Geometry (unrotated frame; the mark is then rotated 30° and scaled):
  stem       x = 12, y 4.5..19.5; blue core 1.1 wide
  right      half hexagon, open at x = 14.25, y 4.5..15.3, 3.118 wide
  left       circular arc,  open at x =  9.75, y 8.7..19.5, 3.118 wide
  inner      each bracket offset inward by INSET, ends under the stem
  cone       from the stem at each bracket's middle, outward along its
             axis of symmetry, half-angle CONE — every line it crosses is cut,
             and the two pieces either side of a cut are mirror images.
"""
import io, math, os, colorsys, cairosvg
from PIL import Image, ImageDraw, ImageFont

LINE = float(os.environ.get("LINE", "0.65"))
PAD = float(os.environ.get("PAD", "0.65"))
SUFFIX = os.environ.get("SUFFIX", "")
CONE = math.radians(float(os.environ.get("CONE", "9")))   # half-angle
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

def cone_y(x, ym):
    """Where the cone's two edges cross the vertical x: (upper, lower)."""
    t = abs(x - STEM_X) * math.tan(CONE)
    return ym - t, ym + t

def hex_pieces(xo, yt, yb, d, w, r, sx=None, inset=0.0):
    """Half hexagon (optionally offset `inset` inward and started at sx),
    cut by the cone on its vertical side into two mirror pieces."""
    h, ym = yb - yt, (yt + yb) / 2
    slope, L = (h / 6) / w, math.hypot(w, h / 6)
    xv = xo + d * (w - inset)
    ox, oy = xo - d * inset * (h / 6) / L, yt + inset * w / L
    top = lambda x: oy + slope * d * (x - ox)
    bot = lambda x: (yb - (oy - yt)) - slope * d * (x - ox)
    x0 = xo if sx is None else sx
    cu, cl = cone_y(xv, ym)
    return [rounded([(x0, top(x0)), (xv, top(xv)), (xv, cu)], r),
            rounded([(xv, cl), (xv, bot(xv)), (x0, bot(x0))], r)], [(x0, top(x0)), (x0, bot(x0))]

def arc_pieces(xo, yt, yb, d, w, sx=None, inset=0.0):
    """Circular arc (the outer circle, or the same centre `inset` smaller),
    cut where the cone crosses it into two mirror pieces."""
    h, ym = yb - yt, (yt + yb) / 2
    R = (h * h / 4 + w * w) / (2 * w)
    cx = xo + d * (w - R)
    r = R - inset
    x0 = xo if sx is None else sx
    half = math.sqrt(max(r * r - (x0 - cx) ** 2, 0.0))
    # Cone edge from (STEM_X, ym) at ±CONE about +d·x; intersect the circle.
    pts = []
    for sgn in (-1, 1):
        ux, uy = d * math.cos(CONE), sgn * math.sin(CONE)
        px, py = STEM_X - cx, 0.0
        b = 2 * (px * ux + py * uy); c = px * px + py * py - r * r
        t = (-b + math.sqrt(b * b - 4 * c)) / 2
        pts.append((STEM_X + ux * t, ym + uy * t))
    sweep = 1 if d > 0 else 0
    (ux_, uy_), (lx, ly) = pts
    a = f"M{f(x0)} {f(ym - half)}A{f(r)} {f(r)} 0 0 {sweep} {f(ux_)} {f(uy_)}"
    b = f"M{f(lx)} {f(ly)}A{f(r)} {f(r)} 0 0 {sweep} {f(x0)} {f(ym + half)}"
    return [a, b], [(x0, ym - half), (x0, ym + half)]

def bracket(kind, nested, xo, yt, yb, d):
    """[(pieces, width, open_ends_to_round)] for the bracket and its inner line."""
    out = []
    p, ends = (hex_pieces(xo, yt, yb, d, W, R_CORNER) if kind == "hex" else arc_pieces(xo, yt, yb, d, W))
    out.append((p, LINE, ends))          # outer open ends show: round them
    if nested:
        sx = STEM_X + d * (CORE / 2 + LINE / 2)   # runs under the stem to the core
        p, _ = (hex_pieces(xo, yt, yb, d, W, R_CORNER * 0.6, sx, INSET) if kind == "hex"
                else arc_pieces(xo, yt, yb, d, W, sx, INSET))
        out.append((p, LINE, []))         # its ends are under the stem
    return out

def draw(pieces, width, ends):
    """Cut ends square (along the cone), open ends round (a dot, since one
    path can't carry two cap styles)."""
    s = "".join(f'<path fill="none" stroke="currentColor" stroke-width="{width}" stroke-linecap="butt" stroke-linejoin="round" d="{p}"/>' for p in pieces)
    s += "".join(f'<circle cx="{f(x)}" cy="{f(y)}" r="{f(width/2)}" fill="currentColor"/>' for x, y in ends)
    return s

def capsule(x, y0, y1, r):
    return (f"M{f(x - r)} {f(y0)}A{f(r)} {f(r)} 0 0 1 {f(x + r)} {f(y0)}"
            f"L{f(x + r)} {f(y1)}A{f(r)} {f(r)} 0 0 1 {f(x - r)} {f(y1)}Z")

def wrap(body, scale="1.25000"):
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">'
            f'<g transform="translate(12 12) scale({scale}) translate(-12 -12)"><g transform="rotate(30 12 12)">'
            + body + '</g></g></svg>')

def white(right, left, nested):
    body = "".join(draw(*b) for b in bracket(right, nested, 14.25, 4.5, 15.3, +1) + bracket(left, nested, 9.75, 8.7, 19.5, -1))
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
    under = draw(*l[0]) + "".join(draw(*b) for b in r[1:])
    warm = draw(*r[0]) + "".join(draw(*b) for b in l[1:]) + stroke("M12 4.5v15", STEM)
    line = stroke("M12 4.9v14.2", CORE)
    return under, warm, line

VARIANTS = {"phiber_arc": ("Arc left", ("hex", "semi", False)),
            "phiber_arc_nested": ("Arc left, nested", ("hex", "semi", True)),
            "phiber_arc_right_nested": ("Arc right, nested", ("semi", "hex", True))}

def png(svg, px):
    return Image.open(io.BytesIO(cairosvg.svg2png(bytestring=svg.encode(), output_width=px, output_height=px))).convert("RGBA")

files = {}
for name, (_, args) in VARIANTS.items():
    n = f"{name}_cut{SUFFIX}"
    files[name] = (white(*args), n)
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
        n = files[name][1]
        svg = open(f"{n}_color.svg").read() if colour else files[name][0].replace("currentColor", "#E8ECF2")
        small = open(f"{n}_color.svg").read() if colour else files[name][0].replace("currentColor", "#1F2328")
        x = pad + i * (cell + pad)
        d.rounded_rectangle([x, y0 + 20, x + cell, y0 + 20 + cell], 18, fill=(22, 24, 28))
        im = png(svg, 240); sheet.paste(im, (x + 30, y0 + 50), im)
        d.rounded_rectangle([x, y0 + 340, x + cell, y0 + 430], 14, fill=(236, 238, 242))
        for j, px in enumerate((64, 32, 24, 16)):
            im = png(small, px); sheet.paste(im, (x + 20 + j * 72, y0 + 385 - px // 2), im)
        d.text((x, y0 + 436), label, fill=(220, 224, 230), font=font)
        d.text((x, y0 + 454), f"{n}{'_color' if colour else ''}.svg", fill=(140, 148, 160), font=font)
sheet.save(f"phiber_mark_cut{SUFFIX}.png")
print("ok", SUFFIX)
