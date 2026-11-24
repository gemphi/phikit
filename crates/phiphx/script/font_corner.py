"""How round a Phi font's corners are, as a share of its capital height:
the corner of a square letter's outline where its straight sides stop
short of the corner (T's top left, L's foot right, Z's top right), as
`data/shape.tsv` rounds the dice's edges by. Run from phikit/crates:
python phiphx/script/font_corner.py
"""
from fontTools.ttLib import TTFont
from fontTools.pens.recordingPen import RecordingPen

FONT = {"Phi-Soft": "phiber/assets/phi-fonts/Phi-Soft/PhiSoft-Regular.otf", "Phiber": "phiber/assets/phi-fonts/Phiber/Phiber-Regular.otf"}
# The letter and which corner: (x side, y side), 0 the low one, 1 the high.
CORNER = [("T", (0, 1)), ("L", (1, 0)), ("Z", (1, 1))]


def outline(path, letter):
    font = TTFont(path)
    pen = RecordingPen()
    font.getGlyphSet()[font.getBestCmap()[ord(letter)]].draw(pen)
    point = []
    for op, args in pen.value:
        if op == "moveTo":
            point = [args[0]]
        elif op == "lineTo":
            point.append(args[0])
        elif op == "closePath":
            break
    return point


for name, path in FONT.items():
    seen = []
    for letter, (sx, sy) in CORNER:
        point = outline(path, letter)
        xs, ys = [p[0] for p in point], [p[1] for p in point]
        cx, cy = (min(xs), max(xs))[sx], (min(ys), max(ys))[sy]
        across = min(abs(p[0] - cx) for p in point if abs(p[1] - cy) < 1)
        down = min(abs(p[1] - cy) for p in point if abs(p[0] - cx) < 1)
        seen.append((across + down) / 2 / (max(ys) - min(ys)))
    print(f"{name}\t{sum(seen) / len(seen):.4f}")
