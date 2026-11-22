"""Phiogy's mark, drawn on the measures of its own face.

    python script/logo.py            (from crates/appkit/app/phiogy)

Reads `data/logo.tsv` and the font it names, and writes the mark into
`asset/icon/phiogy.svg`, a picture in one ink, as every icon is: the
font's letter o as it is drawn, a voice, with a tail leaving its foot at
the left, which makes a speech mark of it. It is the web Phiogy's mark
(`puikit/app/phiogy/mark.py`), on the face this window is set in.

Also writes `doc/logo.html`, a sheet of the mark, large and at the size
of the bar. Every length is a measure of the font times a number from
the table."""
import io
import os

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.recordingPen import RecordingPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

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
        return {'one': 1.0, 'round': right - left, 'stem': side[-1] - side[0], 'thin': dash[3] - dash[1], 'drop': -self.bound('p')[1]}


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
    face = Face(os.path.normpath(os.path.join(HERE, 'data', row.pop('font')[0])))
    measure = face.measure()
    size = {name: float(cell[0]) * measure[cell[1]] for name, cell in row.items()}
    made = {NAME: voice(face, size, size['room'])}
    for name, text in made.items():
        io.open(os.path.join(HERE, 'asset', 'icon', name + '.svg'), 'w', encoding='utf-8', newline='\n').write(text)
    io.open(os.path.join(HERE, 'doc', 'logo.html'), 'w', encoding='utf-8', newline='\n').write(sheet(list(made)))
    print(' '.join('%s=%s' % (name, number(value)) for name, value in measure.items()))


main()
