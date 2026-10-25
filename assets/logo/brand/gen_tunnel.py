"""Welcome backdrop: the Phiber mark as a tunnel.

The mark is the tunnel seen at 60° off: turned flat (stem level, 0°) and
repeated from larger than the screen down to the centre. Each level is the
mark exactly — nothing added — scaled so it sits inside the one before
without touching it (its outer brackets clear the previous level's inner
ones), so lines thin as they recede. Each level is its own file and its own
shade of the sphere colour #3B4657 (base over black at falling opacity),
because GPUI paints an SVG as a one-colour mask.
"""
import math, os, sys
OUT = sys.argv[1] if len(sys.argv) > 1 else "."
# the chosen mark (arc right, nested) in its 24-unit frame, stem at x = 12
MARK = [
    ("M14.250 4.500A6.235 6.235 0 0 1 14.250 15.300", 1.3),
    ("M9.750 8.700L7.844 9.800Q6.632 10.500 6.632 11.900L6.632 16.300Q6.632 17.700 7.844 18.400L9.750 19.500", 1.3),
    ("M10.950 10.692L9.684 11.422Q8.957 11.842 8.957 12.682L8.957 15.518Q8.957 16.358 9.684 16.778L10.950 17.508", 1.0),
    ("M13.050 6.492A3.910 3.910 0 0 1 13.050 13.308", 1.0),
    ("M12 4.5v15", 2.4),
]
# (hex, opacity of #3B4657 over black) — near to far, from the shade table
LEVELS = [("#3B4657", 1.0), ("#353F4E", 0.9), ("#2F3846", 0.8), ("#29313D", 0.7),
          ("#232A34", 0.6), ("#1E232C", 0.5), ("#181C23", 0.4), ("#12151A", 0.3)]
BG = "#0C0E11"            # shade 2: base at 20% over black
K0 = 5.0                  # the nearest level: its stem is wider than the window
WEIGHT = 0.1              # line weight against the icon's: thin lines, so levels can nest closely
# Nesting without touching. The arc and hex sit off-centre along the stem, so
# a smaller level also shifts; measured on the sampled strokes, levels stay
# clear up to a step of ~0.38 and are clearest at 0.34 (gap 0.22 units).
RATIO = 0.34

def level(k):
    body = "".join(f'<path d="{d}" stroke-width="{w * WEIGHT:.3f}"/>' for d, w in MARK)
    # rotate 90: the stem lies level. Centre (12,12) of the mark → (50,50).
    return (f'<g transform="translate(50 50) rotate(90) scale({k:.5f}) translate(-12 -12)" fill="none" '
            f'stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">{body}</g>')

os.makedirs(OUT, exist_ok=True)
for i, (hex_, a) in enumerate(LEVELS):
    k = K0 * RATIO ** i
    svg = ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100">'
           + level(k) + "</svg>")
    open(os.path.join(OUT, f"welcome_tunnel_{i}.svg"), "w").write(svg)
print("ratio", round(RATIO, 4), "levels", len(LEVELS))
