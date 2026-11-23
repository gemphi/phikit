"""The ground the user's mark stands on, drawn on Phi-Soft's own shape.

    python script/soft.py            (from crates/appkit)

Writes `asset/icon/soft-fill.svg`: the outline of the letter o of
Phi-Soft Light, filled, in one ink as every icon is. Nothing in it is
drawn by hand: it is the font's own outer line, turned over (a font's y
runs up, a picture's down) in a square box as high as the letter.
"""
import io
import os

from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FONT = os.path.normpath(os.path.join(HERE, '..', 'phiber', 'assets', 'phi-fonts', 'Phi-Soft', 'PhiSoft-Light.otf'))
LETTER = 'o'


def number(value):
    return ('%.2f' % value).rstrip('0').rstrip('.')


def main():
    font = TTFont(FONT)
    glyph = font.getGlyphSet()
    pen = DecomposingRecordingPen(glyph)
    glyph[font.getBestCmap()[ord(LETTER)]].draw(pen)
    # The letter's outlines, one a list of steps; the widest is its
    # outer line.
    ring = []
    for verb, point in pen.value:
        if verb == 'moveTo':
            ring.append([])
        ring[-1].append((verb, point))
    wide = lambda one: max(x for _, point in one for x, _ in point) - min(x for _, point in one for x, _ in point)
    outer = max(ring, key=wide)
    place = [(x, y) for _, point in outer for x, y in point]
    left, right = min(x for x, _ in place), max(x for x, _ in place)
    low, high = min(y for _, y in place), max(y for _, y in place)
    side = max(right - left, high - low)
    path = SVGPathPen(glyph, ntos=number)
    turned = TransformPen(path, (1, 0, 0, -1, (side - (right + left)) / 2, (side + high + low) / 2))
    for verb, point in outer:
        getattr(turned, verb)(*point)
    text = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 %s %s" fill="currentColor" role="img" aria-label="soft">\n  <path d="%s"/>\n</svg>\n' % (number(side), number(side), path.getCommands())
    io.open(os.path.join(HERE, 'asset', 'icon', 'soft-fill.svg'), 'w', encoding='utf-8', newline='\n').write(text)
    print('soft-fill.svg', number(right - left), 'by', number(high - low))


main()
