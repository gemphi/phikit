"""The Phiber mark: each bracket as two overlapping 2/3 pieces, the inner one stepped in by LINE + GAP and drawn underneath.

Every line — outer and inner — is two parts with rounded ends and a gap
between them. The outer line is 1/3 then 2/3 along its length; the inner line
is the opposite, 2/3 then 1/3, so the short part of one sits against the long
part of the other. The left side mirrors the right through the stem's centre
(the mark's half-turn symmetry), so the right bracket's top matches the left
bracket's bottom. Lengths are measured along the visible line (the inner
line's ends run under the stem and don't count).
"""
import io, math, os, colorsys
import resvg_py
from PIL import Image, ImageDraw, ImageFont

LINE = float(os.environ.get("LINE", "0.65"))
PAD = float(os.environ.get("PAD", "0.65"))
SUFFIX = os.environ.get("SUFFIX", "")
# The visible gap between a line's two parts: the original mark's gap, where
# a bracket's open end stops short of the stem (14.25 − 13.2 − 1.3/2 = 0.4).
GAP = float(os.environ.get("GAP", "0.4"))
CORE = 1.1
STEM = CORE + 2 * PAD
INSET = 2.325
R_CORNER = 1.4
W, STEM_X = 17.368 - 14.25, 12.0

def f(v): return f"{v:.3f}"

def hex_pts(xo, yt, yb, d, w, r, sx=None, inset=0.0, n=24):
    """Half hexagon as dense points (rounded corners as quadratic curves)."""
    h = yb - yt
    slope, L = (h / 6) / w, math.hypot(w, h / 6)
    xv = xo + d * (w - inset)
    ox, oy = xo - d * inset * (h / 6) / L, yt + inset * w / L
    top = lambda x: oy + slope * d * (x - ox)
    bot = lambda x: (yb - (oy - yt)) - slope * d * (x - ox)
    x0 = xo if sx is None else sx
    V = [(x0, top(x0)), (xv, top(xv)), (xv, bot(xv)), (x0, bot(x0))]
    def toward(a, b, dist):
        l = math.hypot(b[0]-a[0], b[1]-a[1]); return (a[0]+(b[0]-a[0])*dist/l, a[1]+(b[1]-a[1])*dist/l)
    pts = [V[0]]
    for i in (1, 2):
        a, b = toward(V[i], V[i-1], r), toward(V[i], V[i+1], r)
        pts.append(a)
        for k in range(1, n):
            t = k / n
            pts.append(((1-t)**2*a[0] + 2*(1-t)*t*V[i][0] + t*t*b[0], (1-t)**2*a[1] + 2*(1-t)*t*V[i][1] + t*t*b[1]))
        pts.append(b)
    pts.append(V[3])
    return pts

def arc_pts(xo, yt, yb, d, w, sx=None, inset=0.0, n=160):
    """Circular arc (or the same centre, `inset` smaller) as dense points."""
    h, ym = yb - yt, (yt + yb) / 2
    R = (h * h / 4 + w * w) / (2 * w)
    cx = xo + d * (w - R)
    r = R - inset
    x0 = xo if sx is None else sx
    a0 = math.atan2(-math.sqrt(max(r*r - (x0-cx)**2, 0)), x0 - cx)
    a1 = math.atan2(+math.sqrt(max(r*r - (x0-cx)**2, 0)), x0 - cx)
    if d > 0:   # through angle 0 (rightward bulge)
        angs = [a0 + (a1 - a0) * k / n for k in range(n + 1)]
    else:       # through angle π (leftward bulge)
        b0, b1 = a0 % (2*math.pi), a1
        angs = [b0 + (b1 - b0) * k / n for k in range(n + 1)]
    return [(cx + r * math.cos(a), ym + r * math.sin(a)) for a in angs]

def cumlen(pts):
    out = [0.0]
    for a, b in zip(pts, pts[1:]): out.append(out[-1] + math.hypot(b[0]-a[0], b[1]-a[1]))
    return out

def sub(pts, cl, s0, s1):
    """The points of the polyline between arc lengths s0 and s1."""
    def at(s):
        for i in range(1, len(cl)):
            if cl[i] >= s:
                t = (s - cl[i-1]) / max(cl[i] - cl[i-1], 1e-9)
                return (pts[i-1][0] + (pts[i][0]-pts[i-1][0])*t, pts[i-1][1] + (pts[i][1]-pts[i-1][1])*t)
        return pts[-1]
    mid = [p for p, c in zip(pts, cl) if s0 < c < s1]
    return [at(s0)] + mid + [at(s1)]

def split(pts, first, hidden):
    """Two parts: `first` of the visible length, then the rest, with a GAP
    between their rounded ends. `hidden` is the stem's half-width: stretches
    of the line closer to the stem than that don't count toward the split."""
    cl = cumlen(pts)
    vis = [c for p, c in zip(pts, cl) if abs(p[0] - STEM_X) >= hidden]
    v0, v1 = vis[0], vis[-1]
    s = v0 + (v1 - v0) * first
    g = (GAP + LINE) / 2           # half the gap, plus a round cap's reach
    return [sub(pts, cl, 0, s - g), sub(pts, cl, s + g, cl[-1])]

def poly(pts):
    return "M" + "L".join(f"{f(x)} {f(y)}" for x, y in pts)

def nearest(pts, cl, p):
    """Arc length of the point on the polyline nearest to p."""
    best, at = 1e9, 0.0
    for a, b, c0, c1 in zip(pts, pts[1:], cl, cl[1:]):
        vx, vy = b[0]-a[0], b[1]-a[1]
        L2 = vx*vx + vy*vy or 1e-12
        t = max(0.0, min(1.0, ((p[0]-a[0])*vx + (p[1]-a[1])*vy) / L2))
        q = (a[0]+vx*t, a[1]+vy*t)
        d2 = (q[0]-p[0])**2 + (q[1]-p[1])**2
        if d2 < best: best, at = d2, c0 + (c1-c0)*t
    return at

def tracks(kind, xo, yt, yb, d, flip):
    off = LINE + GAP
    sx = STEM_X + d * (STEM / 2 + GAP + LINE / 2)     # clear of the stem by GAP
    outer = hex_pts(xo, yt, yb, d, W, R_CORNER) if kind == "hex" else arc_pts(xo, yt, yb, d, W)
    step = (hex_pts(xo, yt, yb, d, W, R_CORNER, sx, off) if kind == "hex"
            else arc_pts(xo, yt, yb, d, W, sx, off))
    if flip:
        outer, step = outer[::-1], step[::-1]
    return outer, step

def half_len(kind, xo, yt, yb, d, flip):
    outer, step = tracks(kind, xo, yt, yb, d, flip)
    cl, cs = cumlen(outer), cumlen(step)
    return nearest(step, cs, sub(outer, cl, cl[-1] / 2, cl[-1] / 2)[0])

def bracket(kind, xo, yt, yb, d, flip, small):
    """Outer 2/3 on the bracket's own track; a smaller piece `small` long on
    the concentric track stepped in by LINE + GAP. Both tracks move together
    along the stem so the smaller piece's stem-side end is level with the
    core's end: the shapes are never bent or pulled apart."""
    outer, step = tracks(kind, xo, yt, yb, d, flip)
    cl, cs = cumlen(outer), cumlen(step)
    top_part = sub(outer, cl, cl[-1] / 3, cl[-1])
    under_part = sub(step, cs, 0.0, small)
    core_end = 4.9 if d > 0 else 19.1
    dy = core_end - under_part[0][1]
    mv = lambda ps: [(x, y + dy) for x, y in ps]
    return [draw_parts([mv(under_part), mv(top_part)])]

def draw_parts(parts):
    return "".join(f'<path fill="none" stroke="currentColor" stroke-width="{LINE}" stroke-linecap="round" stroke-linejoin="round" d="{poly(p)}"/>' for p in parts)

def capsule(x, y0, y1, r):
    return (f"M{f(x - r)} {f(y0)}A{f(r)} {f(r)} 0 0 1 {f(x + r)} {f(y0)}"
            f"L{f(x + r)} {f(y1)}A{f(r)} {f(r)} 0 0 1 {f(x - r)} {f(y1)}Z")

def wrap(body, scale="1.25000"):
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">'
            f'<g transform="translate(12 12) scale({scale}) translate(-12 -12)"><g transform="rotate(30 12 12)">'
            + body + '</g></g></svg>')

RIGHT, LEFT = (14.25, 4.5, 15.3, +1, False), (9.75, 8.7, 19.5, -1, True)

def sides(right, left, nested):
    # one length for both smaller pieces: the shorter of the two halves
    small = min(half_len(right, *RIGHT), half_len(left, *LEFT))
    return (bracket(right, *RIGHT, small), bracket(left, *LEFT, small))

def white(right, left, nested):
    r, l = sides(right, left, nested)
    body = "".join(r + l)
    body += f'<path fill="currentColor" fill-rule="evenodd" d="{capsule(12, 4.5, 19.5, STEM/2)}{capsule(12, 4.9, 19.1, CORE/2)}"/>'
    return wrap(body)

def hsl(h, s, l):
    r, g, b = colorsys.hls_to_rgb(h/360, l/100, s/100); return "#%02X%02X%02X" % (round(r*255), round(g*255), round(b*255))
WARM, COOL = hsl(38, 92, 50), hsl(217, 91, 60)

def stroke(d, w):
    return f'<path fill="none" stroke="currentColor" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round" d="{d}"/>'

def layers(right, left, nested):
    """Bottom to top: cool (left bracket, right inner), warm (right bracket,
    left inner, stem), line (the stem's core)."""
    r, l = sides(right, left, nested)
    under = l[0] + "".join(r[1:])
    warm = r[0] + "".join(l[1:]) + stroke("M12 4.5v15", STEM)
    return under, warm, stroke("M12 4.9v14.2", CORE)

VARIANTS = {"phiber_arc": ("Arc left", ("hex", "semi", False)),
            "phiber_arc_right": ("Arc right", ("semi", "hex", False))}

def png(svg, px, bg=None):
    svg = svg.replace('width="24" height="24"', f'width="{px}" height="{px}"', 1)
    kw = {"background": bg} if bg else {}
    return Image.open(io.BytesIO(bytes(resvg_py.svg_to_bytes(svg_string=svg, **kw)))).convert("RGBA")

for name, (_, args) in VARIANTS.items():
    n = f"{name}_true{SUFFIX}"
    open(f"{n}.svg", "w").write(white(*args))
    under, warm, line = layers(*args)
    open(f"{n}_cool.svg", "w").write(wrap(under, "1.48148"))
    open(f"{n}_warm.svg", "w").write(wrap(warm, "1.48148"))
    open(f"{n}_line.svg", "w").write(wrap(line, "1.48148"))
    open(f"{n}_color.svg", "w").write(wrap(f'<g color="{COOL}">{under}</g><g color="{WARM}">{warm}</g><g color="{COOL}">{line}</g>'))

font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 15)
cell, pad = 300, 24
sheet = Image.new("RGB", (pad + 2 * (cell + pad), 2 * 480 + 10), (30, 32, 36))
d = ImageDraw.Draw(sheet)
for row, colour in enumerate((False, True)):
    y0 = row * 480
    for i, (name, (label, _)) in enumerate(VARIANTS.items()):
        n = f"{name}_true{SUFFIX}"
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
sheet.save(f"phiber_mark_true{SUFFIX}.png")
print("ok", SUFFIX)
