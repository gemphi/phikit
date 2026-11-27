"""The salon's couches, after the user's pictures of 28 November 2026:
gilt frames and buttoned velvet, built as the jets are, of straight
lines and circles.
    python script/salon.py            (from crates/appkit/app/phiogy)
Reads `data/once/salon.tsv`: the outlines as chains of circles, the
measures, the looks and the pieces. Writes the outlines into
`data/once/` (`salon-*.svg`), their names into `source.tsv`, the looks
into `look.tsv`, and the pieces' rows at the foot of `thing.tsv`; no
other row is touched."""
import math
from fractions import Fraction
from chain import box, g, keep, lines, outline, write
HEAD = '# The salon, from data/once/salon.tsv by script/salon.py.'
def read():
    table = {'line': {}, 'measure': {}, 'look': [], 'piece': []}
    for line in lines('salon.tsv'):
        if not line.strip() or line.startswith('#'):
            continue
        cell = line.split('\t')
        match cell[0]:
            case 'line':
                x, y, r = (float(Fraction(v)) for v in cell[3].split())
                table['line'].setdefault(cell[1], []).append(((x, y), r if cell[2] == 'round' else -r))
            case 'measure':
                table['measure'][cell[1]] = [float(v) for v in cell[2].split()]
            case 'look':
                table['look'].append(cell[1:])
            case 'piece':
                table['piece'].append(cell[1:])
    return table
def shrunk(chain, by):
    """The chain drawn in by `by`: each circle round its own middle, a
    round one smaller, a hollow one larger."""
    return [(c, r - by if abs(r) > 1e-9 else r) for c, r in chain]
def inside(p, point, room):
    """Whether `p` is inside the outline's points and at least `room`
    from its edge."""
    n, odd, near = len(point), False, math.inf
    for k in range(n):
        (ax, ay), (bx, by) = point[k], point[(k + 1) % n]
        if (ay > p[1]) != (by > p[1]) and p[0] < ax + (p[1] - ay) * (bx - ax) / (by - ay):
            odd = not odd
        dx, dy = bx - ax, by - ay
        t = max(0.0, min(1.0, ((p[0] - ax) * dx + (p[1] - ay) * dy) / (dx * dx + dy * dy or 1)))
        near = min(near, math.hypot(p[0] - ax - t * dx, p[1] - ay - t * dy))
    return odd and near >= room
def grid(x0, x1, y0, y1, step):
    """Buttons in diamonds over a box: every other row half a step on."""
    a, b = step
    out, j, y = [], 0, y0
    while y <= y1 + 1e-9:
        x = x0 + (a / 2 if j % 2 else 0)
        while x <= x1 + 1e-9:
            out.append((x, y))
            x += a
        j, y = j + 1, y + b / 2
    return out
def centred(x0, x1, y0, y1, step):
    """A grid as wide and high as fits, in the middle of the box."""
    a, b = step
    nx, ny = int((x1 - x0) / a), int((y1 - y0) / (b / 2))
    ox, oy = (x1 - x0 - nx * a) / 2, (y1 - y0 - ny * b / 2) / 2
    return grid(x0 + ox, x0 + ox + nx * a, y0 + oy, y0 + oy + ny * b / 2, step)
class Rows(list):
    def add(self, thing, part, form, size, at, turn='-', round_='-', look='-', source='-', repeat='-'):
        cell = [thing, part, form, size if isinstance(size, str) else ' '.join(g(v) for v in size),
                ' '.join(g(v) for v in at), turn, round_ if isinstance(round_, str) else g(round_), look, source, repeat]
        self.append('\t'.join(cell))
def main():
    t = read()
    m, chains, rows, sources = t['measure'], t['line'], Rows(), []
    gilt, tuck, room, step, button = m['gilt'][0], m['tuck'][0], m['keep'][0], m['tuft'], m['button'][0]
    def drawn(name, rings):
        """An outline of one or more rings (a ring after the first a hole):
        written, and its points and box."""
        path, point = outline(rings[0])
        for hole in rings[1:]:
            path += outline(hole, True)[0]
        write('salon-' + name, path, point, 'The %s of the salon: lines and circles, from data/once/salon.tsv by script/salon.py.' % name)
        sources.append('salon-' + name)
        return point, box(point)
    leg_box = drawn('leg', [chains['leg']])[1]
    leg_high = leg_box[3] - leg_box[1]
    def legs(thing, wide, deep, middle=False):
        xs = [-1, 0, 1] if middle else [-1, 1]
        for sx in xs:
            for sz in (-1, 1):
                turn = math.degrees(math.atan2(-sz, sx))
                rows.add(thing, 'leg', 'outline', (0, leg_high, 0.05), (sx * (wide / 2 - 0.07), leg_high / 2, sz * (deep / 2 - 0.07)), '0 %s 0' % g(turn), 0.15, 'trim', 'salon-leg')
    # A back: its gilt frame round a hole, the velvet in it a little
    # larger and proud of it, buttons on the velvet, a rosette on the
    # crest and a scroll on each shoulder; a thing of its own, its middle
    # its box's, so a piece leans it back.
    def back(kind):
        chain = chains['back-' + kind]
        inner = shrunk(chain, gilt)
        point, (x0, y0, x1, y1) = drawn('back-%s-gilt' % kind, [chain, inner])
        velvet = shrunk(chain, gilt - tuck)
        vpoint, (u0, v0, u1, v1) = drawn('back-%s-velvet' % kind, [velvet])
        mx, my, high = (x0 + x1) / 2, (y0 + y1) / 2, y1 - y0
        name = 'salon-back-' + kind
        rows.add(name, 'gilt', 'outline', (0, high, 0.05), (0, 0, 0), '-', 0.02 / high, 'trim', 'salon-back-%s-gilt' % kind)
        rows.add(name, 'velvet', 'outline', (0, v1 - v0, 0.09), ((u0 + u1) / 2 - mx, my - (v0 + v1) / 2, -0.005), '-', 0.035 / (v1 - v0), 'velvet', 'salon-back-%s-velvet' % kind)
        for x, y in centred(u0, u1, v0, v1, step):
            if inside((x, y), vpoint, room):
                rows.add(name, 'button', 'sphere', (button, button, button * 0.6), (x - mx, my - y, 0.04), look='button')
        crest = max((c for c in chain if c[1] > 0), key=lambda c: -c[0][1] + c[1])
        rows.add(name, 'rosette', 'bloom', (0.11, 0.08, 0.025), (crest[0][0] - mx, my - crest[0][1] + crest[1] - gilt / 2, 0.03), '-', 0.3, 'trim', '6 0.22')
        for c, r in chain:
            if r > 0 and c[1] < (y0 + y1) / 2 and abs(c[0]) > 0.15:
                rows.add(name, 'scroll', 'torus', (0.07, 0.022, 0.07), (c[0] - mx, my - c[1], 0.03), '90 0 0', look='trim')
        return name, high
    # An arm at each end: a velvet pad over the seat's edge, a gilt rail
    # under it, a gilt post down to the seat in front and a scroll at the
    # pad's front.
    def arms(thing, wide, deep, long):
        x, z = wide / 2 - 0.045, -0.04
        front = z + long / 2
        for sx in (-1, 1):
            rows.add(thing, 'pad', 'capsule', (0.09, long, 0.1), (sx * x, 0.64, z), '90 0 0', look='velvet')
            rows.add(thing, 'side', 'box', (0.07, 0.22, long - 0.04), (sx * x, 0.53, z), '-', 0.4, 'velvet')
            rows.add(thing, 'rail', 'capsule', (0.03, long + 0.04, 0.03), (sx * x, 0.595, z), '90 0 0', look='trim')
            rows.add(thing, 'post', 'capsule', (0.045, 0.26, 0.045), (sx * x, 0.5, front - 0.04), '-10 0 0', look='trim')
            rows.add(thing, 'scroll', 'torus', (0.1, 0.032, 0.1), (sx * (x + 0.005), 0.64, front), '90 90 0', look='trim')
    def seated(thing, wide, deep, seats, legs_middle):
        legs(thing, wide, deep, legs_middle)
        rows.add(thing, 'apron', 'box', (wide, 0.1, deep), (0, 0.33, 0), '-', 0.25, 'trim')
        for k in range(seats):
            w = (wide - 0.12) / seats
            # Its top 0.46 high, as every couch's seat is.
            rows.add(thing, 'seat', 'box', (w - 0.01, 0.13, deep - 0.08), (-(wide - 0.12) / 2 + w * (k + 0.5), 0.395, 0.02), '-', 0.5, 'velvet')
        for x in ([0] if seats == 1 else [-wide / 3.4, 0, wide / 3.4]):
            rows.add(thing, 'shell', 'bloom', (0.15, 0.09, 0.03), (x, 0.315, deep / 2 + 0.008), '-', 0.3, 'trim', '7 0.2')
    for kind, wide, deep, seats, long in (('chair', 0.78, 0.72, 1, 0.44), ('sofa', 1.9, 0.8, 3, 0.52)):
        thing = 'salon-' + kind
        name, high = back(kind)
        seated(thing, wide, deep, seats, seats > 1)
        rows.add(thing, 'back', 'thing', '1', (0, 0.4 + high / 2, -deep / 2 + 0.07), '-8 0 0', '-', '-', name)
        arms(thing, wide, deep, long)
    # The chaises: a base, a seat, a raised end over one end and a back
    # along the rear sweeping up into it; buttons on the seat, the back
    # and the end's face; a rosette on the roll.
    end_point, (e0, f0, e1, f1) = drawn('chaise-end', [chains['chaise-end']])
    back_point, (b0, c0, b1, c1) = drawn('chaise-back', [chains['chaise-back']])
    long, deep = 1.9, 0.72
    for kind in ('chaise', 'meridienne'):
        thing = 'salon-' + kind
        if kind == 'chaise':
            for sx in (-1, 1):
                for sz in (-1, 1):
                    rows.add(thing, 'foot', 'sphere', (0.11, 0.09, 0.11), (sx * (long / 2 - 0.1), 0.045, sz * (deep / 2 - 0.1)), '-', '-', 'trim', '8 0.1')
            rows.add(thing, 'base', 'box', (long, 0.2, deep), (0, 0.19, 0), '-', 0.2, 'velvet')
            foot = 0.29
        else:
            legs(thing, long, deep)
            rows.add(thing, 'rail', 'box', (long, 0.08, deep), (0, 0.3, 0), '-', 0.3, 'trim')
            rows.add(thing, 'shell', 'bloom', (0.15, 0.09, 0.03), (0, 0.3, deep / 2 + 0.008), '-', 0.3, 'trim', '7 0.2')
            foot = 0.34
        top = foot + 0.13
        rows.add(thing, 'seat', 'box', (long - 0.06, 0.13, deep - 0.04), (0, foot + 0.065, 0), '-', 0.5, 'velvet')
        shift = long / 2 + 0.03 - e1
        rows.add(thing, 'end', 'outline', (0, f1 - f0, deep - 0.02), (shift + (e0 + e1) / 2, foot - (f0 + f1) / 2, 0), '-', 0.06 / (f1 - f0), 'velvet', 'salon-chaise-end')
        rows.add(thing, 'back', 'outline', (0, c1 - c0, 0.16), ((b0 + b1) / 2, foot - (c0 + c1) / 2, -deep / 2 + 0.08), '-', 0.05 / (c1 - c0), 'velvet', 'salon-chaise-back')
        for x, z in centred(-long / 2 + 0.12, shift + e0 - 0.08, -deep / 2 + 0.24, deep / 2 - 0.12, step):
            rows.add(thing, 'button', 'sphere', (button, button * 0.6, button), (x, top, z), look='button')
        for x, y in centred(b0, b1, c0, c1, step):
            if inside((x, y), back_point, room) and foot - y > top + 0.05 and x < shift + e0:
                rows.add(thing, 'button', 'sphere', (button, button, button * 0.6), (x, foot - y, -deep / 2 + 0.16), look='button')
        for x, y in centred(e0, e1, f0, f1, step):
            if inside((x, y), end_point, room) and foot - y > top + 0.05:
                rows.add(thing, 'button', 'sphere', (button, button, button * 0.6), (shift + x, foot - y, deep / 2 - 0.01), look='button')
        (rx, ry), rr = max((c for c in chains['chaise-end'] if c[1] > 0), key=lambda c: c[1])
        rows.add(thing, 'rosette', 'bloom', (rr * 1.5, rr * 1.5, 0.03), (shift + rx, foot - ry, deep / 2), '-', 0.3, 'trim', '8 0.15')
    for name, kind, velvet, buttons, trim in t['piece']:
        rows.add(name, 'body', 'thing', '1', (0, 0, 0), '-', '-', 'velvet=%s button=%s trim=%s' % (velvet, buttons, trim), 'salon-' + kind)
    named = {row.split('\t')[0] for row in rows}
    text = [line for line in lines('thing.tsv') if line != HEAD and line.split('\t')[0] not in named]
    while text and not text[-1].strip():
        text.pop()
    keep('thing.tsv', '\n'.join(text + [HEAD] + rows) + '\n')
    for table, head, new in (('source.tsv', "# The salon's outlines, from data/once/salon.tsv by script/salon.py.", ['%s\t%s.svg' % (s, s) for s in sources]),
                             ('look.tsv', "# The salon's velvets, from data/once/salon.tsv by script/salon.py.", ['\t'.join(look) for look in t['look']])):
        mine = {line.split('\t')[0] for line in new}
        kept = [line for line in lines(table) if line != head and line.split('\t')[0] not in mine]
        while kept and not kept[-1].strip():
            kept.pop()
        keep(table, '\n'.join(kept + [head] + new) + '\n')
    print('%d pieces, %d outlines, %d rows' % (len(t['piece']), len(sources), len(rows)))
main()
