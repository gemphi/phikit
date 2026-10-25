"""The Welcome tunnel: the icon continued outward in even steps.

The icon's inner and outer layers share a centre and are true offsets of
each other (2.325 apart). Every further layer continues that evenly:
    thickness   t_n = 1.0 + 0.3 n            (the icon: 1.0, 1.3)
    clear gap   G = 1.175 between layers     (the icon's own gap)
    offset      d_{n+1} = d_n + G + (t_n + t_{n+1}) / 2
    arc         one centre (11.133, 9.9), radius 3.912 + d_n
    hex         offset d_n outward, corner radius 0.84 + 0.2409 d_n
    open ends   stay beside the stem, where the icon's outer layer ends
                (arc at x 14.25, hex at x 9.75): every layer meets the stem
So thickness, radius and spacing each grow by a constant amount per layer.
Each layer has its own shade: #3B4657 over black, 100% at the icon falling
evenly to 30% at the edge; the ground is 20%.
"""
import math
BASE = (0x3B, 0x46, 0x57)
BG = "#0C0E11"
T0, DT, G = 1.0, 0.3, 1.175
ARC_C = (11.133, 9.9); ARC_R0 = 3.912; ARC_X0 = 13.05
HEX_CY, HEX_H0, HEX_XV0, HEX_X0, RC0, RC_K = 14.1, 3.408, 8.957, 10.95, 0.84, 0.2409
OPEN_K = 1.2 / 2.325

def thickness(n): return T0 + DT * n
def offset(n):
    d = 0.0
    for i in range(n): d += G + (thickness(i) + thickness(i + 1)) / 2
    return d

def arc(n):
    d = offset(n); r = ARC_R0 + d; x = ARC_X0 if n == 0 else 14.25
    dy = math.sqrt(max(r * r - (x - ARC_C[0]) ** 2, 0))
    large = 1 if x < ARC_C[0] else 0
    return f"M{x:.3f} {ARC_C[1] - dy:.3f}A{r:.3f} {r:.3f} 0 {large} 1 {x:.3f} {ARC_C[1] + dy:.3f}"

def hexa(n):
    d = offset(n)
    # inner hex's top edge: from the open end (x0, cy−h) to the side (xv, cy−h+h/3)
    p0 = (HEX_X0, HEX_CY - HEX_H0); p1 = (HEX_XV0, HEX_CY - HEX_H0 + HEX_H0 / 3)
    ux, uy = p1[0] - p0[0], p1[1] - p0[1]; L = math.hypot(ux, uy); ux, uy = ux / L, uy / L
    nx, ny = uy, -ux                      # outward normal (up and away from the stem)
    if ny > 0: nx, ny = -nx, -ny
    q0 = (p0[0] + nx * d, p0[1] + ny * d)  # offset top-edge line through q0, direction u
    xv = HEX_XV0 - d                      # offset vertical side
    t = (xv - q0[0]) / ux; corner = (xv, q0[1] + uy * t)
    xo = HEX_X0 if n == 0 else 9.75       # open end beside the stem, as the icon's outer layer
    t_end = (xo - q0[0]) / ux; start = (xo, q0[1] + uy * t_end)
    rc = RC0 + RC_K * d
    k = rc * math.tan(math.radians(30) / 1)  # 60° turn at each corner → tangent length rc·tan30°
    k = rc * math.tan(math.radians(30))
    a = (corner[0] - ux * k, corner[1] - uy * k)   # on the top edge, before the corner
    b = (corner[0], corner[1] + k)                 # on the side, after the corner
    mirror = lambda p: (p[0], 2 * HEX_CY - p[1])
    s2, a2, b2 = mirror(start), mirror(a), mirror(b)
    return (f"M{start[0]:.3f} {start[1]:.3f}L{a[0]:.3f} {a[1]:.3f}A{rc:.3f} {rc:.3f} 0 0 0 {b[0]:.3f} {b[1]:.3f}"
            f"L{b2[0]:.3f} {b2[1]:.3f}A{rc:.3f} {rc:.3f} 0 0 0 {a2[0]:.3f} {a2[1]:.3f}L{s2[0]:.3f} {s2[1]:.3f}")

def shade(n, levels):
    a = 1.0 - 0.7 * n / (levels - 1)
    return "#%02X%02X%02X" % tuple(round(v * a) for v in BASE), round(a, 3)

def _g(arc_top, scale, body, cx=50, cy=50):
    rot = -90 if arc_top else 90
    return (f'<g transform="translate({cx} {cy}) rotate({rot}) scale({scale:.4f}) translate(-12 -12)" fill="none" '
            f'stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">{body}</g>')

def layer(n, arc_top, scale):
    w = thickness(n)
    return _g(arc_top, scale, f'<path d="{arc(n)}" stroke-width="{w:.3f}"/><path d="{hexa(n)}" stroke-width="{w:.3f}"/>')

def stem(arc_top, scale):
    return _g(arc_top, scale, '<path d="M12 4.5v15" stroke-width="2.4"/>')

if __name__ == "__main__":
    for n in range(3): print(n, round(offset(n), 3), thickness(n), arc(n), hexa(n), sep="\n  ")
