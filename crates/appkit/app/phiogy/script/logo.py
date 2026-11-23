"""Phiogy's mark, drawn on the measures of its own face.

    python script/logo.py            (from crates/appkit/app/phiogy)

Reads `data/logo.tsv` and the font it names, and writes the mark into
`asset/icon/phiogy.svg`, a picture in one ink, as every icon is. Two
marks can be drawn, and the table's `mark` row says which is written
there: `voice`, the font's letter o with a tail leaving its foot at the
left, which makes a speech mark of it (the web Phiogy's mark,
`puikit/app/phiogy/mark.py`); or `p`, a filled p with two bars cut out
of its bowl (`shape.py`), in the option of `data/logo-corner.tsv` the
row names.

Every drawing is also written into `doc/logo/`, so none is lost when
another is chosen. `doc/logo.html` shows the chosen mark large and at
the size of the bar, and `doc/logo-option.png` every option in every
ink it is tried in (`sheet.py`). `doc/logo/registry-icon.json` is the
chosen p as the web's kit keeps a mark: the `icon` of Phiogy's entry in
`puikit/app/phi-kit/src/registry.json`. Every length is a measure of
the font times a number from the table."""
import io
import json
import os

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.recordingPen import RecordingPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

import shape
import sheet as option

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
NAME = 'phiogy'


def table(path):
    """The rows of a table: name, and the cells after it."""
    out = {}
    for line in io.open(path, encoding='utf-8').read().splitlines():
        if line.strip() and not line.startswith('#'):
            cell = [part.strip() for part in line.split('\t')]
            out[cell[0]] = cell[1:]
    return out


def number(value):
    return ('%.2f' % value).rstrip('0').rstrip('.')


class Face:
    """The font, and what of it the mark is drawn on."""

    def __init__(self, path):
        self.font = TTFont(path)
        self.glyph = self.font.getGlyphSet()
        self.name = self.font.getBestCmap()

    def bound(self, letter):
        pen = BoundsPen(self.glyph)
        self.glyph[self.name[ord(letter)]].draw(pen)
        return pen.bounds

    def path(self, letter, high):
        """A letter's outline as a path, turned over: a font's y runs up
        and a picture's down, from `high`."""
        pen = SVGPathPen(self.glyph, ntos=number)
        self.glyph[self.name[ord(letter)]].draw(TransformPen(pen, (1, 0, 0, -1, 0, high)))
        return pen.getCommands()

    def ring(self, letter):
        """The leftmost place of each of a letter's outlines."""
        pen = RecordingPen()
        self.glyph[self.name[ord(letter)]].draw(pen)
        out = []
        for verb, point in pen.value:
            if verb == 'moveTo':
                out.append([])
            out[-1].extend(x for x, _ in point)
        return sorted(min(one) for one in out if one)

    def measure(self):
        left, low, right, high = self.bound('o')
        dash, side = self.bound('-'), self.ring('o')
        # The o's side is an upright stroke: from its outline's left to
        # its inside's left.
        return {'one': 1.0, 'round': right - left, 'tall': high - low, 'stem': side[-1] - side[0], 'thin': dash[3] - dash[1], 'drop': -self.bound('p')[1]}


def picture(part, box, room, label):
    """A square picture round `box` (left, top, right, bottom), with
    `room` of its side clear on each side."""
    left, top, right, bottom = box
    side = max(right - left, bottom - top) / (1 - 2 * room)
    x, y = (left + right - side) / 2, (top + bottom - side) / 2
    head = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="%s %s %s %s" fill="currentColor" role="img" aria-label="%s">' % (number(x), number(y), number(side), number(side), label)
    return head + '\n' + ''.join('  %s\n' % one for one in part) + '</svg>\n'


def voice(face, size, room):
    """The o, and a tail from its foot at the left."""
    left, low, right, high = face.bound('o')
    bowl = '<path fill-rule="evenodd" d="%s"/>' % face.path('o', high)
    thick, foot = size['tail.thick'], high - low
    # The tail starts inside the bowl's left side and leans out as it
    # drops; half its thickness is its round end.
    start = (left + size['tail.inset'], foot - thick / 2)
    end = (left + size['tail.inset'] - size['tail.lean'], foot + size['tail.drop'] - thick / 2)
    tail = '<path fill="none" stroke="currentColor" stroke-width="%s" stroke-linecap="round" d="M%s %sL%s %s"/>' % (number(thick), number(start[0]), number(start[1]), number(end[0]), number(end[1]))
    return picture([bowl, tail], (min(left, end[0] - thick / 2), 0, right, foot + size['tail.drop']), room, NAME)


def sheet(name):
    """A page that shows the mark large and at the bar's size, on a
    light ground and a dark one."""
    tile = lambda file, ink, ground, side: '<span style="display:inline-grid;place-items:center;width:%spx;height:%spx;margin:8px;border-radius:14px;background:%s;color:%s"><span style="display:block;width:%spx;height:%spx">%s</span></span>' % (side * 1.5, side * 1.5, ground, ink, side, side, io.open(os.path.join(HERE, 'asset', 'icon', file + '.svg'), encoding='utf-8').read())
    row = lambda file: '<p style="font:15px sans-serif;margin:16px 8px 0">%s</p>%s' % (file, ''.join(tile(file, ink, ground, side) for ink, ground in (('#1d1d1b', '#dededa'), ('#f1f1ee', '#191918')) for side in (160, 36, 20)))
    return '<!doctype html>\n<meta charset="utf-8">\n<title>Phiogy mark</title>\n<body style="margin:24px;background:#c9c9c4">\n%s\n</body>\n' % '\n'.join(row(file) for file in name)


def main():
    row = table(os.path.join(HERE, 'data', 'logo.tsv'))
    font = os.path.normpath(os.path.join(HERE, 'data', row.pop('font')[0]))
    face = Face(font)
    measure = face.measure()
    # A row is a length when it names a measure; else it is words.
    word = {name: cell for name, cell in row.items() if len(cell) < 2 or cell[1] not in measure}
    size = {name: float(cell[0]) * measure[cell[1]] for name, cell in row.items() if name not in word}
    corner = table(os.path.join(HERE, 'data', 'logo-corner.tsv'))
    ring = {name: shape.mark(size, cell) for name, cell in corner.items()}
    cut = lambda name: picture(['<path fill-rule="evenodd" d="%s"/>' % shape.path(ring[name][0], number)], ring[name][1], size['room'], NAME)
    made = {'voice': voice(face, size, size['room']), **{'p-' + name: cut(name) for name in ring}}
    os.makedirs(os.path.join(HERE, 'doc', 'logo'), exist_ok=True)
    write = lambda text, *place: io.open(os.path.join(HERE, *place), 'w', encoding='utf-8', newline='\n').write(text)
    for name, text in made.items():
        write(text, 'doc', 'logo', name + '.svg')
    chosen = '-'.join(word['mark'])
    write(made[chosen], 'asset', 'icon', NAME + '.svg')
    write(sheet([NAME]), 'doc', 'logo.html')
    # The mark's two bars alone, as an icon: what a message is marked
    # with, in the place of a star with a plus.
    bars, _ = ring[word['mark'][-1]]
    reach = lambda at: [value for one in bars[1:] for piece in one for value in ([piece[1][at]] if piece[0] == 'point' else [piece[1][at] - piece[2], piece[1][at] + piece[2]])]
    span = (min(reach(0)), min(reach(1)), max(reach(0)), max(reach(1)))
    write(picture(['<path d="%s"/>' % shape.path(bars[1:], number)], span, size['room'], NAME + '-bar'), 'asset', 'icon', NAME + '-bar.svg')
    # The same mark as the web's kit keeps one: a list of paths in a
    # square box, the mark as high as the table says and in its middle.
    rings, box = ring[word['mark'][-1]]
    side, scale = size['web.box'], size['web.high'] / (box[3] - box[1])
    place = (scale, (side - (box[2] - box[0]) * scale) / 2, (side - size['web.high']) / 2)
    write(json.dumps([{'d': shape.path(rings, number, place, True), 'fill': 'currentColor', 'stroke': 'none', 'sw': 0}], indent=2), 'doc', 'logo', 'registry-icon.json')
    dots = {name: ([shape.flat(one, size['p.fine']) for one in rings], box) for name, (rings, box) in ring.items()}
    option.sheet(dots, word, HERE, font, os.path.join(HERE, 'doc', 'logo-option.png'))
    print(chosen, ' '.join('%s=%s' % (name, number(value)) for name, value in measure.items()))


main()
