"""Reads the user's sketch sheets ("Our Design Matters") and writes them as
the Rust tables the footwear is lofted from: the tall boot's side, front
and sole views, and the slip-on's own side view (the low shoe is drawn on
its own last — rounder, higher at the throat — not cut from the boot's),
and the ballet flat's, its ribbons left out."""
import numpy as np
N = 64
def body(s, a, b):
    """The shoe without what the drawing draws above it — the ribbons and the
    ankle they wind round: everything over the line from `a` (the heel
    counter's top) to `b` (the throat) and behind `a` above it is dropped,
    and the largest piece kept."""
    from scipy import ndimage
    ys, xs = np.mgrid[0:s.shape[0], 0:s.shape[1]]
    line = a[1] + (np.clip(xs, a[0], b[0]) - a[0]) * (b[1] - a[1]) / (b[0] - a[0])
    keep = s & ((ys >= line) | (xs > b[0]))
    lab, n = ndimage.label(keep)
    sizes = ndimage.sum(keep, lab, range(1, n + 1))
    return lab == (1 + int(np.argmax(sizes)))
def side(name, toe_left=False, cut=None):
    s = np.load(name + '.npy')
    if cut: s = body(s, *cut)
    if toe_left: s = s[:, ::-1]
    ys, xs = np.nonzero(s); floor, top = ys.max(), ys.min(); H = floor - top
    low = s[int(floor - 0.3*H):floor+1]
    L, heel = 0, 0
    for r in low:
        c = np.nonzero(r)[0]
        if len(c) and c.max() - c.min() > L: L, heel = c.max() - c.min(), c.min()
    rows = []
    for k in range(N + 1):
        y = int(round(floor - k / N * H)); c = np.nonzero(s[y])[0]
        rows.append(((c.min() - heel) / L, (c.max() - heel) / L))
    bottom = []
    for k in range(N + 1):
        x = int(round(heel + k / N * L)); c = np.nonzero(s[:, min(x, s.shape[1]-1)])[0]
        bottom.append((floor - c.max()) / L)
    return H / L, rows, bottom
def front(height):
    s = np.load('tall_front.npy'); ys, xs = np.nonzero(s); floor, top = ys.max(), ys.min(); H = floor - top
    out = []
    for k in range(N + 1):
        y = int(round(floor - k / N * H)); c = np.nonzero(s[y])[0]
        out.append((c.max() - c.min()) / 2 / H * height)
    return out
def sole():
    s = np.load('tall_sole.npy'); ys, xs = np.nonzero(s); tip, heel = ys.min(), ys.max(); L = heel - tip
    mid = (xs.min() + xs.max()) / 2
    out = []
    for k in range(N + 1):
        y = int(round(heel - k / N * L)); c = np.nonzero(s[min(y, s.shape[0]-1)])[0]
        out.append(((c.max() - c.min()) / 2 / L, ((c.max() + c.min()) / 2 - mid) / L))
    return out
H, rows, bottom = side('tall_side')
SH, srows, sbottom = side('slip_side', toe_left=True)
# The ballet flat: its body under the ribbons, from the heel counter's top to the throat.
FH, frows, fbottom = side('flat_side', cut=((85, 240), (500, 330)))
fr = front(H)
so = sole()
f = lambda x: f"{x:.4}"
def outline(name, what, h, rows, bottom):
    return [f"/// {what}", f"pub const {name}: BOutline = BOutline {{",
            f"    height: {f(h)},",
            "    side: [" + ", ".join(f"({f(a)}, {f(b)})" for a, b in rows) + "],",
            "    underside: [" + ", ".join(f(x) for x in bottom) + "],", "};"]
src = ["//! The footwear, read off the user's sketch sheets (\"Our Design Matters\")",
       "//! by `tools/sketch/emit.py`. Everything is in foot lengths: heights from",
       "//! the floor, distances along the foot from the back of the heel.",
       "", f"pub const SAMPLES: usize = {N + 1};", "",
       "/// A shoe as one drawing's side view gives it.",
       "pub struct BOutline {",
       "    /// How tall it is drawn.",
       "    pub height: f32,",
       "    /// At each of `SAMPLES` heights from the floor to `height`: its back and its front.",
       "    pub side: [(f32, f32); SAMPLES],",
       "    /// From the heel to the toe: how high its underside is off the floor — the heel's and the toe's rounding, the arch.",
       "    pub underside: [f32; SAMPLES],",
       "}", ""] + outline("TALL", "The 3/4 tall boot, side (outer): the last every boot is cut from.", H, rows, bottom) + [""] + outline("SLIP", "The slip-on shoe, side, drawn toe to the left (read turned round).", SH, srows, sbottom) + [""] + outline("FLAT", "The ballet flat, side: its body, the ribbons and the ankle above it left out.", FH, frows, fbottom) + ["",
       "/// At the tall boot's heights: its half width, seen from the front.",
       "pub const FRONT: [f32; SAMPLES] = [" + ", ".join(f(x) for x in fr) + "];",
       "/// From the heel to the toe: the sole's half width and how far its middle sits off the foot's line, seen from below.",
       "pub const TREAD: [(f32, f32); SAMPLES] = [" + ", ".join(f"({f(a)}, {f(b)})" for a, b in so) + "];", ""]
open('/home/claude/ws/crates/phiplay/src/build/footwear/sketch.rs', 'w').write("\n".join(src))
print(H, SH, FH, frows[::8], fbottom[::8])
