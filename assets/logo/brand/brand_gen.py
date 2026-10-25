"""Phiber brand assets: the official mark (rounded half-hex bowls, 30° slant, unbroken
two-tone stem) and the PHIBER letterforms built from the same vocabulary."""
import math, os, zipfile
from shapely.geometry import Polygon, box
OUT = "/mnt/user-data/outputs/phiber-brand"
BLUE, ORANGE = "#5B9CF0", "#F28C28"
DARK, LIGHT = "#161B22", "#F3F5F8"
os.makedirs(f"{OUT}/app", exist_ok=True); os.makedirs(f"{OUT}/font", exist_ok=True)

def f(v): return f"{v:.3f}"

# ---------------------------------------------------------------- mark
half_stem, gap, band, cr, RJ, TILT = 1.2, 0.4, 1.3, 0.6, 1.4, 30
edge = half_stem + gap
H = 10.8; TH = math.radians(30); L = H / (2 + 2 * math.sin(TH)); W = L * math.cos(TH)
cy_top, cy_bot = 9.9, 14.1

def rounded(p, r, radii=None):
    """Polyline with interior corners filleted by r (or radii[i] per corner; 0 = plain join)."""
    out = [f"M{f(p[0][0])} {f(p[0][1])}"]
    for i in range(1, len(p) - 1):
        a, b, c = p[i - 1], p[i], p[i + 1]
        if radii is not None and radii[i] == 0:
            out.append(f"L{f(b[0])} {f(b[1])}"); continue
        u = (a[0] - b[0], a[1] - b[1]); v = (c[0] - b[0], c[1] - b[1])
        lu, lv = math.hypot(*u), math.hypot(*v); u = (u[0] / lu, u[1] / lu); v = (v[0] / lv, v[1] / lv)
        ang = math.acos(max(-1, min(1, u[0] * v[0] + u[1] * v[1]))); t = r / math.tan(ang / 2)
        p1 = (b[0] + u[0] * t, b[1] + u[1] * t); p2 = (b[0] + v[0] * t, b[1] + v[1] * t)
        sw = 1 if (u[0] * v[1] - u[1] * v[0]) < 0 else 0
        out.append(f"L{f(p1[0])} {f(p1[1])}A{f(r)} {f(r)} 0 0 {sw} {f(p2[0])} {f(p2[1])}")
    out.append(f"L{f(p[-1][0])} {f(p[-1][1])}")
    return "".join(out)

def hex_pts(sx, cy, side, h, sw_, reach_gap):
    """Centreline of a half-hex bowl beside a stem at sx (stroke sw_), opening toward the stem."""
    l = h / (2 + 2 * math.sin(TH)); w = l * math.cos(TH)
    x0 = sx + side * (reach_gap + sw_ / 2); x1 = x0 + side * w
    return [(x0, cy - h / 2), (x1, cy - l), (x1, cy + l), (x0, cy + h / 2)]

def stroke(d, col, w, extra=""):
    return f'<path fill="none" stroke="{col}" stroke-width="{w}" stroke-linecap="round" stroke-linejoin="round"{extra} d="{d}"/>'

def c_hex(cy, side, col):
    return stroke(rounded(hex_pts(12, cy, side, H, band, edge), RJ), col, band)

def inner_hex(cy, side, col):
    p = hex_pts(12, cy, side, H, band, edge)
    closed = Polygon(p + [(12, cy + H / 2), (12, cy - H / 2)])
    inner = closed.buffer(-(band / 2 + gap), join_style="mitre")
    r2 = RJ - band / 2 - gap
    inner = inner.buffer(-r2, join_style="mitre").buffer(r2, join_style="round", resolution=8)
    xc = 12 + side * edge
    keep = box(xc, -50, xc + 50, 50) if side > 0 else box(xc - 50, -50, xc, 50)
    shape = inner.intersection(keep).buffer(-cr, join_style="mitre").buffer(cr, join_style="round", resolution=8)
    d = "M" + "L".join(f"{f(x)} {f(y)}" for x, y in shape.exterior.coords) + "Z"
    return f'<path fill="{col}" d="{d}"/>'

def stem(outer, inner):
    return stroke("M12 4.5v15", outer, 2.4) + stroke("M12 4.9v14.2", inner, 1.1)

def mark_layers(warm, cool):
    """Official colourway: top bowl warm C / cool inner; bottom cool C / warm inner; stem warm/cool."""
    warm_l = c_hex(cy_top, +1, warm) + inner_hex(cy_bot, -1, warm) + stroke("M12 4.5v15", warm, 2.4)
    cool_l = inner_hex(cy_top, +1, cool) + c_hex(cy_bot, -1, cool) + stroke("M12 4.9v14.2", cool, 1.1)
    return warm_l, cool_l

def wrap(body, bg=None, tilt=TILT, size=512, vb="0 0 24 24", fill=1.0):
    bx = f'<rect width="24" height="24" rx="5.4" fill="{bg}"/>' if bg else ""
    if tilt: body = f'<g transform="rotate({tilt:g} 12 12)">{body}</g>'
    # Grow the art about the canvas centre, keeping the shared 24-unit canvas.
    if fill != 1.0: body = f'<g transform="translate(12 12) scale({fill:.5f}) translate(-12 -12)">{body}</g>'
    w = vb.split()[2]; h = vb.split()[3]
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}" width="{size}" height="{int(size * float(h) / float(w))}">{bx}{body}</svg>')

warm_l, cool_l = mark_layers(ORANGE, BLUE)
open(f"{OUT}/mark.svg", "w").write(wrap(warm_l + cool_l))
open(f"{OUT}/mark-dark.svg", "w").write(wrap(warm_l + cool_l, DARK))
open(f"{OUT}/mark-light.svg", "w").write(wrap(warm_l + cool_l, LIGHT))
mw, mc = mark_layers("currentColor", "currentColor")
# app layers: GPUI paints one colour per svg, so the mark is two files stacked.
# In-app sizing. On the 24-unit canvas the tilted mark spans only x 4.06-19.93,
# y 4.30-19.69 — two thirds of it — so a "28 px" mark drew at about 18 px, and
# its 0.4-unit gaps fell under half a pixel and smeared. The art is scaled up
# about the centre, still on the shared 24-unit canvas every icon uses: the
# two-tone brand mark until it fills the canvas (16.2 units with a hair for
# anti-aliasing), so the size the app asks for is the size it gets; the mono
# icon only to ordinary icon padding (19.2), so it sits level with the other
# icons in a title bar.
MARK_FILL = 24 / 16.2
ICON_FILL = 24 / 19.2
open(f"{OUT}/app/phiber_warm.svg", "w").write(wrap(mw, size=24, fill=MARK_FILL))
open(f"{OUT}/app/phiber_cool.svg", "w").write(wrap(mc, size=24, fill=MARK_FILL))
mono_w, mono_c = mark_layers("currentColor", "none")
open(f"{OUT}/app/phiber.svg", "w").write(wrap(mono_w + c_hex(cy_bot, -1, "currentColor") + inner_hex(cy_top, +1, "currentColor"), size=24, fill=ICON_FILL))
open(f"{OUT}/mark-mono.svg", "w").write(wrap(mono_w + c_hex(cy_bot, -1, "currentColor") + inner_hex(cy_top, +1, "currentColor")))

# ---------------------------------------------------------------- letters
# One weight (2.0), cell 24 high, letters 4.5..19.5, every detached piece one gap (0.4)
# from its neighbour, half-hex bowls with the mark's rounded corners.
# A stem is the mirror of B's right side with the middle edge removed: one tall half-hex
# opening toward the bowls. `HEX_STEMS` lists the letters that get it; the rest keep a bar.
import sys
SW, G, R = 2.0, 0.4, 1.3
# Letters stand exactly as tall as the tilted mark's silhouette (4.3 → 19.7 on the canvas).
TOP, BOT = 4.3 + SW / 2, 19.7 - SW / 2
# The letters' "middle" — where bowls meet, where bars cross — is the stem's midpoint.
MID_F = 1 / 2
Y_MID = TOP + (BOT - TOP) * MID_F
P_H = 8.0 * (BOT - TOP) / 15.0             # P's bowl, scaled with the letter height
B_TOP = Y_MID - G / 2 - TOP                # B's upper bowl height
B_BOT = BOT - (Y_MID + G / 2)              # B's lower bowl height
HEX_STEMS = set(sys.argv[1]) if len(sys.argv) > 1 else set("E")
def cap(x0, x1, y): return stroke(f"M{f(x0)} {f(y)}H{f(x1)}", "currentColor", SW)
def bar(x, y0=TOP, y1=BOT): return stroke(f"M{f(x)} {f(y0)}V{f(y1)}", "currentColor", SW)
def bowl(sx, cy, h, side=+1, col="currentColor"):
    return stroke(rounded(hex_pts(sx, cy, side, h, SW, SW / 2 + G), R), col, SW)
def hex_w(h):
    l = h / (2 + 2 * math.sin(TH)); return l, l * math.cos(TH)
def bowl_reach(h): return G + SW + hex_w(h)[1]

def stem(x, ch):
    """Returns (svg, x of the stem's cap edge line). A hex stem opens to the right:
    its caps sit at x, the body bulges left by the half-hex width."""
    if ch not in HEX_STEMS:
        return bar(x), x
    l, w = hex_w(B_TOP)                    # B's bowl slants, facing the other way; only the bar grows
    dy = l * math.sin(TH)
    pts = [(x, TOP), (x - w, TOP + dy), (x - w, BOT - dy), (x, BOT)]
    return stroke(rounded(pts, R), "currentColor", SW), x

def left_extent(ch):  # how far a stem reaches left of its cap line (centreline)
    return hex_w(B_TOP)[1] * E_SLANT if ch in HEX_STEMS else 0.0

def glyph_P(x, accent=None):
    h = P_H; st, sx = stem(x, "P")
    return st + bowl(sx, TOP + h / 2, h, col=accent or "currentColor"), sx + bowl_reach(h) + SW / 2
def glyph_B(x):
    st, sx = stem(x, "B")
    return (st + bowl(sx, TOP + B_TOP / 2, B_TOP) + bowl(sx, BOT - B_BOT / 2, B_BOT),
            sx + bowl_reach(B_TOP) + SW / 2)
def glyph_R(x):
    """B whose bottom bowl does not close: its outer side runs straight down to the baseline."""
    st, sx = stem(x, "R"); l, w = hex_w(B_BOT)
    x0 = sx + G + SW; y0 = Y_MID + G / 2
    leg = stroke(rounded([(x0, y0), (x0 + w, y0 + l * math.sin(TH)), (x0 + w, BOT)], R), "currentColor", SW)
    return st + bowl(sx, TOP + B_TOP / 2, B_TOP) + leg, sx + bowl_reach(B_TOP) + SW / 2
def glyph_H(x):
    """Two stems and the crossbar; the left stem's foot and the right stem's head bend
    outward with P's bend (same slant length and angle, same corner radius)."""
    w = H_W; l, bw = hex_w(P_H); dy = l * math.sin(TH)
    # Bends turn inward, toward the crossbar, at P's angle: the foot of the left stem
    # and the head of the right stem each end as the P bowl's slant does.
    left = stroke(rounded([(x, TOP), (x, BOT - dy), (x + bw, BOT)], R), "currentColor", SW)
    right = stroke(rounded([(x + w, BOT), (x + w, TOP + dy), (x + w - bw, TOP)], R), "currentColor", SW)
    # Nothing touches: the crossbar clears each stem by the one gap, like everything else.
    return left + right + cap(x + SW + G, x + w - SW - G, Y_MID), x + w + SW / 2
def glyph_I(x): return stem(x, "I")[0], x + SW / 2
E_W = hex_w(B_TOP)[1] + 1.8 + SW                        # E, outer edge to outer edge
H_CROSS = 2.34                                          # H's crossbar, edge to edge (settled by eye)
H_W = SW + 2 * G + H_CROSS                              # stems spread so the bar clears both by one gap
H_BAR = 1.8                                # the short bar E borrows (E's caps and middle)
E_SLANT = 1.0                              # E slants: B angle, run trimmed so the merged corners read as B's free caps do
E_MIDF = 3 / 4                             # E's middle bar: a quarter shorter than its caps' bars
def glyph_E(x):
    """E is the mirrored B-bowl stem with its bar stretched; the H crossbar is merged into the
    top and bottom caps and repeated, detached, in the middle beside the stretched bar."""
    l, w = hex_w(B_TOP); l *= E_SLANT; w *= E_SLANT
    dy = l * math.sin(TH)                 # a slant of length l drops l·sin30, as in every bowl
    pts = [(x + H_BAR, TOP), (x, TOP), (x - w, TOP + dy), (x - w, BOT - dy), (x, BOT), (x + H_BAR, BOT)]
    # The slant starts where B's cap sits, unfilleted, so the E's top is B's top mirrored;
    # the bar meets it with the stroke's own round join. Only the bends get the corner radius.
    body = stroke(rounded(pts, R, radii=[0, 0, R, R, 0, 0]), "currentColor", SW)
    x0 = x - w + SW + G                   # one gap off the stretched bar, like H's crossbar off its stem
    x1 = x + H_BAR
    return body + cap(x0, x0 + (x1 - x0) * E_MIDF, Y_MID), x + H_BAR + SW / 2

GLYPHS = {"P": glyph_P, "H": glyph_H, "I": glyph_I, "B": glyph_B, "E": glyph_E, "R": glyph_R}
TRACK = 2.6

def wordmark(text="PHIBER", accent=None):
    x = SW / 2 + 0.5; body = ""
    for i, ch in enumerate(text):
        x += left_extent(ch)
        piece, adv = GLYPHS[ch](x, accent) if (ch == "P" and i == 0) else GLYPHS[ch](x)
        body += piece; x = adv + TRACK
    return body, x - TRACK + 0.5

for ch, fn in GLYPHS.items():
    x = SW / 2 + 0.5 + left_extent(ch); piece, adv = fn(x); w = adv + SW / 2 + 0.5
    open(f"{OUT}/font/{ch}.svg", "w").write(
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {f(w)} 24" width="{int(w / 24 * 512)}" height="512" fill="none" stroke="currentColor">{piece}</svg>')
body, width = wordmark()
open(f"{OUT}/wordmark.svg", "w").write(
    f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {f(width)} 24" width="{int(width / 24 * 512)}" height="512" fill="none" stroke="currentColor">{body}</svg>')
# The in-app wordmark is cropped to its letters (4.3-19.7, plus the same hair),
# so its height is its letter height and it can be set level with the mark.
WORD_Y, WORD_H = 4.2, 15.6
open(f"{OUT}/app/phiber_wordmark.svg", "w").write(
    f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 {f(WORD_Y)} {f(width)} {f(WORD_H)}" fill="none" stroke="currentColor">{body}</svg>')
def colour_word(col): return body.replace("currentColor", col)
def lockup(bg, text_col):
    mk = f'<g transform="rotate(30 12 12)">{warm_l}{cool_l}</g>'
    wm = f'<g transform="translate(26 0)">{colour_word(text_col)}</g>'
    tot = 26 + width
    bx = f'<rect width="{f(tot)}" height="24" fill="{bg}"/>' if bg else ""
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {f(tot)} 24" width="{int(tot / 24 * 256)}" height="256">{bx}{mk}{wm}</svg>'
open(f"{OUT}/lockup.svg", "w").write(lockup(None, "currentColor"))
open(f"{OUT}/lockup-dark.svg", "w").write(lockup(DARK, LIGHT))
open(f"{OUT}/lockup-light.svg", "w").write(lockup(LIGHT, DARK))
print("ok width", round(width, 2), "hex stems", "".join(sorted(HEX_STEMS)))

# ---------------------------------------------------------------- previews
import cairosvg
from PIL import Image
def png(svg_path, out, w):
    cairosvg.svg2png(url=svg_path, write_to=out, output_width=w)
for n in ("mark-dark", "mark-light", "lockup-dark", "lockup-light"):
    png(f"{OUT}/{n}.svg", f"{OUT}/{n}.png", 1024 if n.startswith("lockup") else 512)
# sheet
tiles = [Image.open(f"{OUT}/{n}.png").convert("RGBA") for n in ("mark-dark", "mark-light")]
lk = [Image.open(f"{OUT}/{n}.png").convert("RGBA") for n in ("lockup-dark", "lockup-light")]
Wd = max(1064, 40 + 2 * 552); Ht = 40 + 512 + 40 + lk[0].height + 20 + lk[1].height + 40
sheet = Image.new("RGBA", (Wd, Ht), (40, 46, 56, 255))
sheet.paste(tiles[0], (40, 40), tiles[0]); sheet.paste(tiles[1], (592, 40), tiles[1])
y = 40 + 512 + 40
sheet.paste(lk[0], (40, y), lk[0]); y += lk[0].height + 20; sheet.paste(lk[1], (40, y), lk[1])
sheet.save(f"{OUT}/preview.png")
with zipfile.ZipFile(f"{OUT}/phiber-brand.zip", "w", zipfile.ZIP_DEFLATED) as z:
    for root, _, files in os.walk(OUT):
        for fn in files:
            if fn.endswith((".svg", ".png")) and fn != "preview.png":
                p = os.path.join(root, fn); z.write(p, os.path.relpath(p, OUT))
