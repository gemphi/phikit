"""Symmetric tunnel: the icon replicated level by level about one centre.

The icon's arc and hex sit 2.1 either side of centre along the stem; here
both are centred on the stem's midpoint so each level is mirror-symmetric
and the levels nest evenly. Flat (stem level); arc on top or on the bottom.
"""
import os, sys, json
import numpy as np
from svgpathtools import parse_path
ARC = [("M14.250 4.500A6.235 6.235 0 0 1 14.250 15.300", 1.3), ("M13.050 6.492A3.910 3.910 0 0 1 13.050 13.308", 1.0)]
HEX = [("M9.750 8.700L7.844 9.800Q6.632 10.500 6.632 11.900L6.632 16.300Q6.632 17.700 7.844 18.400L9.750 19.500", 1.3),
       ("M10.950 10.692L9.684 11.422Q8.957 11.842 8.957 12.682L8.957 15.518Q8.957 16.358 9.684 16.778L10.950 17.508", 1.0)]
STEM = ("M12 4.5v15", 2.4)
SHIFT = {"arc": 2.1, "hex": -2.1}          # centre each bracket on the stem's midpoint
LEVEL_HEX = ["#3B4657", "#353F4E", "#2F3846", "#29313D", "#232A34", "#1E232C", "#181C23", "#12151A"]
BG = "#0C0E11"

def group(paths, dy, w):
    return "".join(f'<path transform="translate(0 {dy})" d="{d}" stroke-width="{sw * w:.3f}"/>' for d, sw in paths)

def level(k, arc_top, w):
    rot = -90 if arc_top else 90          # arc (right of stem) → up, or → down
    body = group(ARC, SHIFT["arc"], w) + group(HEX, SHIFT["hex"], w) + f'<path d="{STEM[0]}" stroke-width="{STEM[1] * w:.3f}"/>'
    return (f'<g transform="translate(50 50) rotate({rot}) scale({k:.5f}) translate(-12 -12)" fill="none" '
            f'stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">{body}</g>')

def sampled(w):
    parts = []
    for paths, dy in ((ARC, 2.1), (HEX, -2.1)):
        for d, sw in paths:
            p = parse_path(d); t = np.linspace(0, 1, 300)
            parts.append((np.array([[p.point(x).real - 12, p.point(x).imag - 12 + dy] for x in t]), sw * w))
    return parts

def max_ratio(w):
    parts = sampled(w)
    best = 0
    for r in np.arange(0.40, 0.95, 0.01):
        gap = min(np.sqrt(((a[:, None] - b[None] * r) ** 2).sum(-1)).min() - (wa + wb * r) / 2
                  for a, wa in parts for b, wb in parts)
        if gap > 0.15 * w * 10: best = r
        else: break
    return round(best, 2)

if __name__ == "__main__":
    W = 0.1
    r = max_ratio(W)
    print("max clear ratio", r)
    json.dump({"ratio": r}, open("sym_ratio.json", "w"))
