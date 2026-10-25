import math, cairosvg
from PIL import Image, ImageDraw, ImageFont

import os
# One line weight for every bracket line, outer and inner, equal to the stem's
# edge (its orange padding either side of the blue core), so the lines read
# as the stem's edge carried round the brackets.
STEM_LINE = 1.1                                   # the stem's blue core
LINE = float(os.environ.get("LINE", "0.65"))      # every bracket line
STEM_PAD = float(os.environ.get("PAD", "0.65"))   # the stem's edge, each side
OUTER = INNER = LINE
STEM = STEM_LINE + 2 * STEM_PAD
SUFFIX = os.environ.get("SUFFIX", "")
# Where the inner line runs: its centreline sits this far in from the outer
# one (0.65 + 1.1 + 0.575, set when the inner line was 1.15). Held fixed so
# thinning the inner line keeps its radius; the gap becomes 2.325 − 0.65 − 0.5.
INSET = 2.325
R_CORNER = 1.4

def f(v): return f"{v:.3f}"

def hex_path(xo, yt, yb, d, w, r):
    """Half hexagon open toward the stem at x=xo: (xo,yt) → (xo+dw, yt+h/6) →
    (xo+dw, yb-h/6) → (xo,yb), corners rounded by r (the original mark's)."""
    h = yb - yt
    pts = [(xo, yt), (xo + d*w, yt + h/6), (xo + d*w, yb - h/6), (xo, yb)]
    out = f"M{f(pts[0][0])} {f(pts[0][1])}"
    for i in (1, 2):
        p0, p1, p2 = pts[i-1], pts[i], pts[i+1]
        def toward(a, b, dist):
            L = math.hypot(b[0]-a[0], b[1]-a[1]); return (a[0]+(b[0]-a[0])*dist/L, a[1]+(b[1]-a[1])*dist/L)
        a = toward(p1, p0, r); b = toward(p1, p2, r)
        out += f"L{f(a[0])} {f(a[1])}Q{f(p1[0])} {f(p1[1])} {f(b[0])} {f(b[1])}"
    out += f"L{f(pts[3][0])} {f(pts[3][1])}"
    return out

def semi_path(xo, yt, yb, d, w):
    """A circular arc (one radius, constant curvature) from (xo, yt) to
    (xo, yb) that bulges exactly `w` outward — the half-hexagon's own height
    and width. Radius from chord c and sagitta w: R = (c²/4 + w²) / 2w."""
    c = yb - yt
    radius = (c * c / 4 + w * w) / (2 * w)
    sweep = 1 if d > 0 else 0
    return f"M{f(xo)} {f(yt)}A{f(radius)} {f(radius)} 0 0 {sweep} {f(xo)} {f(yb)}"

def shape(kind, xo, yt, yb, d, w, r):
    return hex_path(xo, yt, yb, d, w, r) if kind == "hex" else semi_path(xo, yt, yb, d, w)

STEM_X = 12.0

def capsule(x, y0, y1, r):
    """A vertical stroke from y0 to y1 with round caps, as a closed outline —
    the shape `stroke-width = 2r, stroke-linecap = round` draws."""
    return (f"M{f(x - r)} {f(y0)}A{f(r)} {f(r)} 0 0 1 {f(x + r)} {f(y0)}"
            f"L{f(x + r)} {f(y1)}A{f(r)} {f(r)} 0 0 1 {f(x - r)} {f(y1)}Z")

def rounded(pts, r):
    """A polyline with every inner corner rounded by r."""
    out = f"M{f(pts[0][0])} {f(pts[0][1])}"
    for i in range(1, len(pts) - 1):
        p0, p1, p2 = pts[i-1], pts[i], pts[i+1]
        def toward(a, b, dist):
            L = math.hypot(b[0]-a[0], b[1]-a[1]); return (a[0]+(b[0]-a[0])*dist/L, a[1]+(b[1]-a[1])*dist/L)
        a = toward(p1, p0, r); b = toward(p1, p2, r)
        out += f"L{f(a[0])} {f(a[1])}Q{f(p1[0])} {f(p1[1])} {f(b[0])} {f(b[1])}"
    return out + f"L{f(pts[-1][0])} {f(pts[-1][1])}"

def inner_path(kind, xo, yt, yb, d, w, sx):
    """The inner line: the bracket offset inward by `delta`, so the gap
    between the two strokes is GAP everywhere, cut where it meets the stem
    (x = sx)."""
    delta = INSET
    ym, h = (yt + yb) / 2, yb - yt
    if kind == "semi":
        # Same centre as the outer circle, radius shorter by delta.
        R = (h * h / 4 + w * w) / (2 * w)
        cx = xo + d * (w - R)
        r = R - delta
        half = math.sqrt(max(r * r - (sx - cx) ** 2, 0.0))
        sweep = 1 if d > 0 else 0
        return f"M{f(sx)} {f(ym - half)}A{f(r)} {f(r)} 0 0 {sweep} {f(sx)} {f(ym + half)}"
    # Half hexagon: its vertical side moves in by delta; its 30° sides move
    # in by delta along their normals, and keep their slope.
    xv = xo + d * (w - delta)
    slope = (h / 6) / w                      # rise per unit out, 30° sides
    L = math.hypot(w, h / 6)
    # Top side: through (xo, yt) toward (xo+dw, yt+h/6); inward normal points
    # toward the middle: (-d*(h/6), w)/L.
    ox, oy = xo - d * delta * (h / 6) / L, yt + delta * w / L
    def top_y(x):
        return oy + slope * d * (x - ox)
    ox2, oy2 = ox, yb - (oy - yt)
    def bot_y(x):
        return oy2 - slope * d * (x - ox2)
    pts = [(sx, top_y(sx)), (xv, top_y(xv)), (xv, bot_y(xv)), (sx, bot_y(sx))]
    return rounded(pts, R_CORNER * 0.6)

def side(kind, nested, xo, yt, yb, d, w, scale=0.5, meet=None):
    """One bracket and, when nested, the same shape inside it, a parallel
    line GAP in from it, its open ends meeting the stem."""
    paths = [(shape(kind, xo, yt, yb, d, w, R_CORNER), OUTER)]
    if nested:
        # The inner line runs under the stem (drawn after it, so the stem
        # covers the end) and stops where its round cap meets the centre
        # line — so it never shows through the line.
        sx = meet if meet is not None else STEM_X + d * (STEM_LINE / 2 + INNER / 2)
        paths.append((inner_path(kind, xo, yt, yb, d, w, sx), INNER))
    return paths

W = 17.368 - 14.25          # the half-hexagon's width
H = 15.3 - 4.5               # its height
def mark(right, left, nested=False):
    paths = side(right, nested, 14.25, 4.5, 15.3, +1, W) + side(left, nested, 9.75, 8.7, 19.5, -1, W)
    body = "".join(f'<path fill="none" stroke="currentColor" stroke-width="{sw}" stroke-linecap="round" stroke-linejoin="round" d="{d}"/>' for d, sw in paths)
    # The stem, with the mark's centre line cut through it — the thin line
    # `phiber_cool.svg` lays down the stem (1.1 wide, `M12 4.9v14.2`). Drawn
    # as one filled shape with a hole (even-odd), so the one-colour icon keeps
    # the line on any background and every renderer draws it the same.
    body += f'<path fill="currentColor" fill-rule="evenodd" stroke="none" d="{capsule(12, 4.5, 19.5, STEM / 2)}{capsule(12, 4.9, 19.1, STEM_LINE / 2)}"/>'
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">'
            '<g transform="translate(12 12) scale(1.25000) translate(-12 -12)"><g transform="rotate(30 12 12)">'
            + body + '</g></g></svg>')

variants = {
    "phiber_arc":            ("Arc left, same height & width", mark("hex", "semi")),
    "phiber_arc_nested":     ("Arc left, nested", mark("hex", "semi", nested=True)),
    "phiber_arc_right_nested": ("Arc right, nested", mark("semi", "hex", nested=True)),
}
for name, (_, svg) in variants.items():
    open(f"{name}{SUFFIX}.svg", "w").write(svg)

def raster(svg, px, fg):
    return Image.open(__import__("io").BytesIO(cairosvg.svg2png(bytestring=svg.replace("currentColor", fg).encode(), output_width=px, output_height=px))).convert("RGBA")

# Contact sheet: each variant large on dark and light, plus 32 / 24 / 16 px.
cell, pad = 300, 24
sheet = Image.new("RGB", (pad + len(variants)*(cell+pad), 560), (30, 32, 36))
d = ImageDraw.Draw(sheet)
try: font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 15)
except Exception: font = ImageFont.load_default()
for i, (name, (label, svg)) in enumerate(variants.items()):
    x = pad + i*(cell+pad)
    d.rounded_rectangle([x, 20, x+cell, 20+cell], 18, fill=(22, 24, 28))
    big = raster(svg, 240, "#E8ECF2"); sheet.paste(big, (x+30, 50), big)
    d.rounded_rectangle([x, 340, x+cell, 440], 14, fill=(236, 238, 242))
    for j, px in enumerate((64, 32, 24, 16)):
        im = raster(svg, px, "#1F2328"); sheet.paste(im, (x+20+j*72, 390-px//2), im)
    d.text((x, 460), label, fill=(220, 224, 230), font=font)
    d.text((x, 485), name + SUFFIX + ".svg", fill=(140, 148, 160), font=font)
sheet.save(f"phiber_mark_variants{SUFFIX}.png")
print("ok")

# ── Colour: the app's two-tone mark ─────────────────────────────────────────
# `atom/brand.rs::mark` stacks two one-colour layers: `phiber_warm` (right
# bracket + stem) in `colors.brand_warm`, `phiber_cool` (left bracket + the
# stem's centre line) in `colors.brand_cool` (`theme/style/colors.rs`).
import colorsys
def hsl(h, s, l):
    r, g, b = colorsys.hls_to_rgb(h / 360, l / 100, s / 100)
    return "#%02X%02X%02X" % (round(r * 255), round(g * 255), round(b * 255))
WARM, COOL = hsl(38, 92, 50), hsl(217, 91, 60)
APP_SCALE = "1.48148"   # the layers' own crop, as phiber_warm/cool.svg

def stroke(d, sw):
    return f'<path fill="none" stroke="currentColor" stroke-width="{sw}" stroke-linecap="round" stroke-linejoin="round" d="{d}"/>'

def layers(right, left, nested):
    """Three layers, bottom to top, so the inner lines sit under the stem:
      under: the cool bracket (left) and the cool inner line (right),
      warm:  the warm bracket (right), the warm inner line (left), the stem,
      line:  the stem's cool centre line.
    Warm and cool alternate by level — each bracket's inner line is the
    other colour, as the stem's centre line is."""
    r = side(right, nested, 14.25, 4.5, 15.3, +1, W)
    l = side(left, nested, 9.75, 8.7, 19.5, -1, W)
    under = stroke(*l[0]) + "".join(stroke(*p) for p in r[1:])
    warm = stroke(*r[0]) + "".join(stroke(*p) for p in l[1:]) + stroke("M12 4.5v15", STEM)
    line = stroke("M12 4.9v14.2", STEM_LINE)
    return under, warm, line

def wrap(body, scale):
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">'
            f'<g transform="translate(12 12) scale({scale}) translate(-12 -12)"><g transform="rotate(30 12 12)">'
            + body + '</g></g></svg>')

color_variants = {
    "phiber_arc":              ("Arc left", ("hex", "semi", False)),
    "phiber_arc_nested":       ("Arc left, nested", ("hex", "semi", True)),
    "phiber_arc_right_nested": ("Arc right, nested", ("semi", "hex", True)),
}
for name, (_, args) in color_variants.items():
    under, warm, line = layers(*args)
    # App-ready layers (one colour each, tinted by the theme at draw time),
    # stacked cool → warm → cool line.
    open(f"{name}{SUFFIX}_cool.svg", "w").write(wrap(under, APP_SCALE))
    open(f"{name}{SUFFIX}_warm.svg", "w").write(wrap(warm, APP_SCALE))
    open(f"{name}{SUFFIX}_line.svg", "w").write(wrap(line, APP_SCALE))
    # A full-colour file for previews and anywhere outside the app.
    color = f'<g color="{COOL}">{under}</g><g color="{WARM}">{warm}</g><g color="{COOL}">{line}</g>'
    open(f"{name}{SUFFIX}_color.svg", "w").write(wrap(color, "1.25000"))

def raster_raw(svg, px):
    return Image.open(__import__("io").BytesIO(cairosvg.svg2png(bytestring=svg.encode(), output_width=px, output_height=px))).convert("RGBA")

sheet = Image.new("RGB", (pad + len(color_variants)*(cell+pad), 560), (30, 32, 36))
d = ImageDraw.Draw(sheet)
for i, (name, (label, _)) in enumerate(color_variants.items()):
    svg = open(f"{name}{SUFFIX}_color.svg").read()
    x = pad + i*(cell+pad)
    d.rounded_rectangle([x, 20, x+cell, 20+cell], 18, fill=(22, 24, 28))
    big = raster_raw(svg, 240); sheet.paste(big, (x+30, 50), big)
    d.rounded_rectangle([x, 340, x+cell, 440], 14, fill=(236, 238, 242))
    for j, px in enumerate((64, 32, 24, 16)):
        im = raster_raw(svg, px); sheet.paste(im, (x+20+j*72, 390-px//2), im)
    d.text((x, 460), label, fill=(220, 224, 230), font=font)
    d.text((x, 485), name + SUFFIX + "_color.svg", fill=(140, 148, 160), font=font)
d.text((pad, 525), f"warm {WARM} (brand_warm)   ·   cool {COOL} (brand_cool)   ·   app layers, bottom to top: <name>_cool + _warm + _line", fill=(140, 148, 160), font=font)
sheet.save(f"phiber_mark_color{SUFFIX}.png")
print("color ok", WARM, COOL)
