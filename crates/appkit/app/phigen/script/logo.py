"""Phigen's mark, drawn on the measures of its own face.

    python script/logo.py            (from crates/appkit/app/phigen)

Reads `data/logo.tsv` and the font it names, and writes two marks into
`asset/icon/`, each a picture in one ink, as every icon is:

  phigen-bar.svg    two lying bars, the upper twice the lower: a P
  phigen-chat.svg   the font's o as a square bubble with a beak under its
                    left corner, and the same two lines inside it

and `doc/logo.html`, a sheet of both, large and at the size of the bar.
Every length is a measure of the font times a number from the table."""
import io
import os
import sys

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.recordingPen import RecordingPen
from fontTools.ttLib import TTFont

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


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
    """The font, and what of it a mark is drawn on."""

    def __init__(self, path):
        self.font = TTFont(path)
        self.glyph = self.font.getGlyphSet()
        self.name = self.font.getBestCmap()

    def bound(self, letter):
        pen = BoundsPen(self.glyph)
        self.glyph[self.name[ord(letter)]].draw(pen)
        return pen.bounds

    def ring(self, letter):
        """A letter's outlines, each a list of points. The face is drawn
        in straight pieces alone; anything else stops the run."""
        pen = RecordingPen()
        self.glyph[self.name[ord(letter)]].draw(pen)
        out = []
        for verb, point in pen.value:
            if verb == 'moveTo':
                out.append([point[0]])
            elif verb == 'lineTo':
                out[-1].append(point[0])
            elif verb not in ('closePath', 'endPath'):
                sys.exit('the letter %s has a %s: this script draws straight pieces' % (letter, verb))
        return out

    def measure(self):
        left, low, right, high = self.bound('o')
        inside = sorted(self.ring('o'), key=len)
        hole = [min(x for x, _ in ring) for ring in inside], [max(x for x, _ in ring) for ring in inside]
        bar = sorted({y for ring in self.ring('=') for _, y in ring})
        dash = self.bound('-')
        return {
            'one': 1.0,
            'round': right - left,
            'stem': max(hole[0]) - left,
            'inside': min(hole[1]) - max(hole[0]),
            'thin': dash[3] - dash[1],
            'pitch': (bar[-1] + bar[-1] - (dash[3] - dash[1])) / 2 - (bar[0] + bar[0] + (dash[3] - dash[1])) / 2,
            'drop': -self.bound('p')[1],
        }


def pill(x, top, long, thick):
    """A lying bar with round ends, its top left at (x, top), y down."""
    return '<rect x="%s" y="%s" width="%s" height="%s" rx="%s"/>' % tuple(number(v) for v in (x, top, long, thick, thick / 2))


def picture(part, box, room, label):
    """A square picture round `box` (left, top, right, bottom), with
    `room` of its side clear on each side."""
    left, top, right, bottom = box
    side = max(right - left, bottom - top) / (1 - 2 * room)
    x, y = (left + right - side) / 2, (top + bottom - side) / 2
    head = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="%s %s %s %s" fill="currentColor" role="img" aria-label="%s">' % (number(x), number(y), number(side), number(side), label)
    return head + '\n' + ''.join('  %s\n' % one for one in part) + '</svg>\n'


def bar(size, room):
    """Two lying bars, the upper the longer: a P."""
    long, thick, gap = size['bar.long'], size['bar.thick'], size['bar.gap']
    part = [pill(0, 0, long, thick), pill(0, thick + gap, long * size['bar.share'], thick)]
    return picture(part, (0, 0, long, thick * 2 + gap), room, 'phigen')


def chat(face, size, room):
    """The o as a bubble, a beak under its left corner, two lines in it.
    The font's y goes up; the picture's goes down from the o's top."""
    left, low, right, high = face.bound('o')
    flip = lambda ring: 'M' + 'L'.join('%s %s' % (number(x), number(high - y)) for x, y in ring) + 'Z'
    bubble = '<path fill-rule="evenodd" d="%s"/>' % ''.join(flip(ring) for ring in face.ring('o'))
    # The beak: down the bubble's left side, to a point under its foot,
    # and back up to the foot. Its corners are as soft as a bar's end.
    soft = size['beak.soft']
    point = [(left + soft, low + size['beak.root']), (left + soft, low - size['beak.drop'] + soft), (left + size['beak.wide'], low + soft)]
    beak = '<path stroke="currentColor" stroke-width="%s" stroke-linejoin="round" d="%s"/>' % (number(soft * 2), flip(point))
    # The lines: in the bubble's inside, from its left, round its middle.
    thick, pitch = size['line.thick'], size['line.pitch']
    start = left + face.measure()['stem'] + size['line.inset']
    long = face.measure()['inside'] - size['line.inset'] * 2
    middle = (high - low) / 2
    line = [pill(start, middle - pitch / 2 - thick / 2, long, thick), pill(start, middle + pitch / 2 - thick / 2, long * size['line.share'], thick)]
    return picture([bubble, beak] + line, (left, 0, right, high - low + size['beak.drop']), room, 'phigen')


def sheet(name):
    """A page that shows each mark large and at the bar's size, on a
    light ground and a dark one."""
    tile = lambda file, ink, ground, side: '<span style="display:inline-grid;place-items:center;width:%spx;height:%spx;margin:8px;border-radius:14px;background:%s;color:%s"><span style="display:block;width:%spx;height:%spx">%s</span></span>' % (side * 1.5, side * 1.5, ground, ink, side, side, io.open(os.path.join(HERE, 'asset', 'icon', file + '.svg'), encoding='utf-8').read())
    row = lambda file: '<p style="font:15px sans-serif;margin:16px 8px 0">%s</p>%s' % (file, ''.join(tile(file, ink, ground, side) for ink, ground in (('#1d1d1b', '#dededa'), ('#f1f1ee', '#191918')) for side in (160, 36, 20)))
    return '<!doctype html>\n<meta charset="utf-8">\n<title>Phigen mark</title>\n<body style="margin:24px;background:#c9c9c4">\n%s\n</body>\n' % '\n'.join(row(file) for file in name)


def main():
    row = table(os.path.join(HERE, 'data', 'logo.tsv'))
    face = Face(os.path.normpath(os.path.join(HERE, 'data', row.pop('font')[0])))
    measure = face.measure()
    size = {name: float(cell[0]) * measure[cell[1]] for name, cell in row.items()}
    made = {'phigen-bar': bar(size, size['room']), 'phigen-chat': chat(face, size, size['room'])}
    for name, text in made.items():
        io.open(os.path.join(HERE, 'asset', 'icon', name + '.svg'), 'w', encoding='utf-8', newline='\n').write(text)
    io.open(os.path.join(HERE, 'doc', 'logo.html'), 'w', encoding='utf-8', newline='\n').write(sheet(list(made)))
    print(' '.join('%s=%s' % (name, number(value)) for name, value in measure.items()))


main()
