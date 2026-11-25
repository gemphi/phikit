"""Draw the house's icons from their measures (asset/icon*.tsv): each a
stroke of one width with round ends, led from point to point, a corner
at each point as round as its radius says. Writes asset/icon/NAME.svg
for every name the table has; the svg files are what comes out, the
table is what is true. An icon the table does not have yet is left as
it is (the older ones are lucide's; they go as the table grows).

    python script/icon.py                  draws every icon of the table
    python script/icon.py --sheet out.png  and a sheet of them to look at

Nothing here is a size or a shape: every one is a row of the table."""
import glob
import math
import os
import sys

KIND = ('run', 'ring', 'fill', 'pill', 'cut', 'hole')
HERE = os.path.dirname(os.path.abspath(__file__))
KIT = os.path.dirname(HERE)
OUT = os.path.join(KIT, 'asset', 'icon')


def table(paths):
    """The table's measures, and each icon's parts in order: the
    table is every `asset/icon*.tsv`, read as one."""
    measure, icon = {}, {}
    for line in (line for path in paths for line in open(path, encoding='utf-8')):
        cell = line.rstrip('\n').split('\t')
        if line.startswith('#') or len(cell) < 3:
            continue
        number = [group.split() for group in cell[2].split(',')]
        if cell[1] in KIND:
            icon.setdefault(cell[0], []).append((cell[1], [[float(n) for n in group] for group in number]))
        else:
            measure[cell[0]] = float(cell[2])
    # Every corner at the family's radius nearest the one written: a
    # small corner, a corner or the frame's; from `round` up, an arc kept
    # as it is. A pill's numbers are its size, not corners.
    family = [measure[name] for name in ('small', 'corner', 'frame') if name in measure]
    whole = measure.get('round', float('inf'))
    def snap(r):
        if r <= 0 or r >= whole or not family:
            return r
        return min(family, key=lambda one: abs(one - r))
    for parts in icon.values():
        for kind, points in parts:
            if kind in ('run', 'ring', 'fill', 'cut'):
                for point in points:
                    if len(point) == 3:
                        point[2] = snap(point[2])
    return measure, icon


def route(point, closed):
    """A route as steps: ('M', p), ('L', p), ('A', r, sweep, p). Each
    corner is cut back along both of its sides by what its radius
    needs, never more than half a side, and joined by an arc."""
    count = len(point)
    corner = range(count) if closed else range(1, count - 1)
    cut = {}
    for at in corner:
        (ax, ay, _), (px, py, r), (bx, by, _) = point[at - 1], point[at], point[(at + 1) % count]
        one, two = math.hypot(ax - px, ay - py), math.hypot(bx - px, by - py)
        ux, uy, vx, vy = (ax - px) / one, (ay - py) / one, (bx - px) / two, (by - py) / two
        turn = math.acos(max(-1.0, min(1.0, ux * vx + uy * vy)))
        if r <= 0 or turn > math.pi - 1e-6:
            continue
        back = min(r / math.tan(turn / 2), one / 2, two / 2)
        r = back * math.tan(turn / 2)
        sweep = 1 if ux * vy - uy * vx < 0 else 0
        cut[at] = ((px + ux * back, py + uy * back), (px + vx * back, py + vy * back), r, sweep)
    step = []
    for at in range(count):
        start, end, r, sweep = cut.get(at, ((point[at][0], point[at][1]), None, 0, 0))
        step.append(('M' if at == 0 else 'L', start))
        if end:
            step.append(('A', r, sweep, end))
    return step


def path(step, closed):
    said = []
    for one in step:
        if one[0] == 'A':
            said.append('A%s %s 0 0 %d %s %s' % (num(one[1]), num(one[1]), one[2], num(one[3][0]), num(one[3][1])))
        else:
            said.append('%s%s %s' % (one[0], num(one[1][0]), num(one[1][1])))
    return ' '.join(said) + (' Z' if closed else '')


def num(value):
    return ('%.3f' % value).rstrip('0').rstrip('.')


def pill(box):
    x, y, wide, high = box
    r = min(wide, high) / 2
    return [(x, y, r), (x + wide, y, r), (x + wide, y + high, r), (x, y + high, r)]


def parts(icon):
    """Each part as (route's steps, closed, filled, what is cut out
    of it): a cut or a hole belongs to the fill before it."""
    made = []
    for kind, number in icon:
        point = pill(number[0]) if kind in ('pill', 'hole') else [tuple(group) for group in number]
        closed = kind != 'run'
        if kind in ('cut', 'hole'):
            made[-1][3].append(route(point, True))
            continue
        made.append((route(point, closed), closed, kind in ('fill', 'pill'), []))
    return made


def svg(icon, measure):
    side, wide = num(measure['box']), num(measure['stroke'])
    drawn = []
    for step, closed, filled, cut in parts(icon):
        more = ' fill="currentColor" fill-rule="evenodd" stroke="none"' if filled else ''
        whole = ' '.join([path(step, closed)] + [path(one, True) for one in cut])
        drawn.append('<path d="%s"%s/>' % (whole, more))
    head = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 %s %s" fill="none" stroke="currentColor" stroke-width="%s" stroke-linecap="round" stroke-linejoin="round">' % (side, side, wide)
    return head + ''.join(drawn) + '</svg>\n'


def dots(step, closed, fine=0.05):
    """A route as dots close together, to draw a sheet with."""
    out, at = [], None
    for one in step:
        if one[0] in 'ML':
            end = one[1]
            if at:
                far = max(1, int(math.hypot(end[0] - at[0], end[1] - at[1]) / fine))
                out.extend((at[0] + (end[0] - at[0]) * k / far, at[1] + (end[1] - at[1]) * k / far) for k in range(1, far + 1))
            else:
                out.append(end)
            at = end
            continue
        _, r, sweep, end = one
        chord = math.hypot(end[0] - at[0], end[1] - at[1])
        high = math.sqrt(max(r * r - chord * chord / 4, 0.0))
        mx, my = (at[0] + end[0]) / 2, (at[1] + end[1]) / 2
        nx, ny = -(end[1] - at[1]) / chord, (end[0] - at[0]) / chord
        side = 1 if sweep else -1
        cx, cy = mx + nx * high * side, my + ny * high * side
        a0, a1 = math.atan2(at[1] - cy, at[0] - cx), math.atan2(end[1] - cy, end[0] - cx)
        span = (a1 - a0) % (2 * math.pi) if sweep else -((a0 - a1) % (2 * math.pi))
        far = max(2, int(abs(span) * r / fine))
        out.extend((cx + r * math.cos(a0 + span * k / far), cy + r * math.sin(a0 + span * k / far)) for k in range(1, far + 1))
        at = end
    if closed:
        far = max(1, int(math.hypot(out[0][0] - at[0], out[0][1] - at[1]) / fine))
        out.extend((at[0] + (out[0][0] - at[0]) * k / far, at[1] + (out[0][1] - at[1]) * k / far) for k in range(1, far + 1))
    return out


def sheet(icons, measure, out, cell=96, across=13):
    from PIL import Image, ImageDraw
    name = sorted(icons)
    rows = -(-len(name) // across)
    page = Image.new('RGB', (cell * across, cell * rows), '#e9e9e4')
    pen = ImageDraw.Draw(page)
    big = (cell - 24) / measure['box']
    wide = measure['stroke'] * big
    for index, one in enumerate(name):
        ox, oy = (index % across) * cell + 12, (index // across) * cell + 12
        for step, closed, filled, cut in parts(icons[one]):
            dot = [(ox + x * big, oy + y * big) for x, y in dots(step, closed)]
            if filled:
                pen.polygon(dot, fill='#16161a')
                for hole in cut:
                    pen.polygon([(ox + x * big, oy + y * big) for x, y in dots(hole, True)], fill='#e9e9e4')
                continue
            for x, y in dot:
                pen.ellipse((x - wide / 2, y - wide / 2, x + wide / 2, y + wide / 2), fill='#16161a')
    page.save(out)


if __name__ == '__main__':
    measure, icons = table(sorted(glob.glob(os.path.join(KIT, 'asset', 'icon*.tsv'))))
    for name, icon in icons.items():
        with open(os.path.join(OUT, name + '.svg'), 'w', encoding='utf-8', newline='\n') as out:
            out.write(svg(icon, measure))
    args = sys.argv[1:]
    if '--sheet' in args:
        sheet(icons, measure, args[args.index('--sheet') + 1])
    print('drew', len(icons), 'icons into', os.path.relpath(OUT, KIT))
