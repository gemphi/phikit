"""The scroll couches, from the user's drawing of a couch's frame, built
as the jets are: straight lines and circles only.

    python script/couch.py            (from crates/appkit/app/phiogy)

Reads `data/once/couch.tsv`: the back's measures, its lattices, the
arms as chains of circles, the post's, the leg's and the seat's parts,
and the couches. Writes each back (one a number of seats and a
lattice), each arm and each band of the lounge chairs as an outline
into `data/once/` (`scroll-*.svg`, `couch-*.svg`),
their names into `source.tsv`, and the couches' rows at the foot of
`thing.tsv`; no other row is touched."""
from fractions import Fraction

from chain import box, g, keep, lines, number, outline, write

HEAD = '# The scroll couches, from data/once/couch.tsv by script/couch.py.'


def read():
    """The table's measures, lattices, arms, parts and couches."""
    table = {'measure': {}, 'open': {}, 'line': {}, 'part': [], 'couch': []}
    for line in lines('couch.tsv'):
        if not line.strip() or line.startswith('#'):
            continue
        cell = line.split('\t')
        match cell[0]:
            case 'measure':
                table['measure'][cell[1]] = [float(Fraction(v)) for v in cell[2].split()]
            case 'open':
                table['open'].setdefault(cell[1], []).append([float(v) for v in cell[2].split()])
            # An outline's circles, and its counter's.
            case 'line' | 'hole':
                value = [float(Fraction(v)) for v in cell[3].split()]
                circle = {'round': lambda x, y, r: ((x, y), r), 'hollow': lambda x, y, r: ((x, y), -r)}[cell[2]](*value)
                table['line'].setdefault(cell[1], {'line': [], 'hole': []})[cell[0]].append(circle)
            case 'part':
                table['part'].append(cell[1:])
            case 'couch' if cell[1] != 'name':
                table['couch'].append(cell[1:])
    return table


def hole(x0, y0, x1, y1, r):
    """A rounded box taken against the clock, a hole in the ring it is in."""
    r = min(r, (x1 - x0) / 2, (y1 - y0) / 2)
    n = number
    return ('M%s %s' % (n(x1 - r), n(y0)) + 'L%s %s' % (n(x0 + r), n(y0)) + 'A%s %s 0 0 0 %s %s' % (n(r), n(r), n(x0), n(y0 + r))
            + 'L%s %s' % (n(x0), n(y1 - r)) + 'A%s %s 0 0 0 %s %s' % (n(r), n(r), n(x0 + r), n(y1))
            + 'L%s %s' % (n(x1 - r), n(y1)) + 'A%s %s 0 0 0 %s %s' % (n(r), n(r), n(x1), n(y1 - r))
            + 'L%s %s' % (n(x1), n(y0 + r)) + 'A%s %s 0 0 0 %s %s' % (n(r), n(r), n(x1 - r), n(y0)) + 'Z')


def ring(m, seats):
    """The back's outline: its lobes' corners, and round each neck the
    lobes' edges dipping in, at the top and at the foot."""
    a, top, foot, c = seats * m['seat'][0] / 2 + m['end'][0], -m['top'][0], -m['foot'][0], m['corner'][0]
    mid, neck, dip, sag = (top + foot) / 2, m['neck'][0], m['dip'][0], m['sag'][0]
    necks = [-a + k * 2 * a / seats for k in range(1, seats)]
    chain = [((-a + c, top + c), c)]
    for x in necks:
        chain += [((x - dip, top + c), c), ((x, mid - neck - sag), -sag), ((x + dip, top + c), c)]
    chain += [((a - c, top + c), c), ((a - c, foot - c), c)]
    for x in reversed(necks):
        chain += [((x + dip, foot - c), c), ((x, mid + neck + sag), -sag), ((x - dip, foot - c), c)]
    chain.append(((-a + c, foot - c), c))
    return chain, a


def openings(m, pattern, seats, a):
    """Each lobe's openings, as boxes: the largest at the couch's end,
    smaller towards a neck; a lobe between two necks has its largest in
    its middle, a lobe between two ends its largest at both."""
    high = m['top'][0] - m['foot'][0] - 2 * m['inset'][0]
    mid = -(m['top'][0] + m['foot'][0]) / 2
    gap, spare = m['gap'][0], m['spare'][0]
    wide = 2 * a / seats
    out = []
    for j in range(seats):
        left, right = j == 0, j == seats - 1
        clear = lambda end: m['clear'][0] if end else m['clear'][1]
        start = -a + j * wide + clear(left)
        room = wide - clear(left) - clear(right)

        def order(k):
            first = pattern[:k]
            match (left, right):
                case (True, True):
                    return first + first[::-1]
                case (True, False):
                    return first
                case (False, True):
                    return first[::-1]
                case _:
                    return first[1:][::-1] + first[:1] + first[1:]

        natural = lambda seq: sum(w * high for w, _, _ in seq) + gap * (len(seq) - 1)
        seq = next((order(k) for k in range(len(pattern), 0, -1) if natural(order(k)) <= room * spare), [])
        if not seq:
            continue
        size = natural(seq)
        scale, extra = min(1.0, room / size), max(0.0, room - size) / (len(seq) + 1)
        x = start + extra
        for w, h, r in seq:
            x1 = x + w * high * scale
            y0, y1 = mid - h * high / 2, mid + h * high / 2
            out.append((x, y0, x1, y1, r * min(x1 - x, y1 - y0) / 2))
            x = x1 + gap * scale + extra
    return out


def main():
    table = read()
    m, sources, rows, made = table['measure'], [], [], set()
    depth, back, front, edge = m['depth'][0], m['back'][0], m['front'][0], m['edge'][0]
    top, foot = m['top'][0], m['foot'][0]
    # The arms and the bands, one outline each, its counter cut out of
    # it; where each stands is a couch's or a chair's.
    arm = {}
    for kind, chain in table['line'].items():
        path, point = outline(chain['line'])
        path += outline(chain['hole'], True)[0] if chain['hole'] else ''
        name = 'couch-' + kind
        write(name, path, point, 'The %s of the couches: lines and circles, from data/once/couch.tsv by script/couch.py.' % kind)
        sources.append(name)
        arm[kind] = (chain['line'][0], box(point))
    # The post, the leg and a bench of each length.
    seats_of = sorted({int(c[1]) for c in table['couch']})
    for part in table['part']:
        for n in (seats_of if part[0] == 'bench' else [0]):
            cell = [part[0] + ('-%d' % n if n else '')] + part[1:]
            fill = {'{wide}': g(n * m['seat'][0] - m['base'][0]), '{first}': g(-(n - 1) * m['seat'][0] / 2), '{repeat}': '-' if n == 1 else '%d %s 0 0' % (n, g(m['seat'][0]))}
            text = '\t'.join(cell)
            for key, value in fill.items():
                text = text.replace(key, value)
            rows.append(text)
    for name, seats, kind, lattice, frame, cushion, trim in table['couch']:
        seats = int(seats)
        chain, a = ring(m, seats)
        back_name = 'scroll-back-%d-%s' % (seats, lattice)
        if back_name not in made:
            path, point = outline(chain)
            path += ''.join(hole(*o) for o in openings(m, table['open'][lattice], seats, a))
            write(back_name, path, point, 'The back of a scroll couch of %d seats, its lattice %s: lines and circles, from data/once/couch.tsv by script/couch.py.' % (seats, lattice))
            sources.append(back_name)
            made.add(back_name)
        high = top - foot
        rows.append('\t'.join([name, 'back', 'outline', '0 %s %s' % (g(high), g(depth)), '0 %s %s' % (g((top + foot) / 2), g(back)), '-', g(edge * depth / high), frame, back_name, '-']))
        # Each arm along its side, from the post behind to the leg in front,
        # seen from the side (turned as a letter side panel is); its ring
        # at the horn's tip, turned with it.
        ((tx, ty), tr), (x0, y0, x1, y1) = arm['arm-' + kind]
        wide = m['arm'][0]
        for side in (-1, 1):
            rows.append('	'.join([name, 'arm', 'outline', '0 %s %s' % (g(y1 - y0), g(wide)), '%s %s %s' % (g(side * a), g(-(y0 + y1) / 2), g(back + (x0 + x1) / 2)), '0 -90 0', g(edge * wide / (y1 - y0)), frame, 'couch-arm-' + kind, '-']))
            rows.append('	'.join([name, 'ring', 'letter', '0 %s %s' % (g(m['ring'][0] * tr), g(m['ring'][1] * wide)), '%s %s %s' % (g(side * a), g(-ty), g(back + tx)), '0 -90 0', '0.03', trim, 'pill-bold:o', '-']))
        swap = 'frame=%s trim=%s' % (frame, trim)
        for part, z in (('couch-post', back), ('couch-leg', front)):
            rows.append('\t'.join([name, part[6:], 'thing', '1', '%s 0 %s' % (g(-a), g(z)), '-', '-', swap, part, '2 %s 0 0' % g(2 * a)]))
        rows.append('\t'.join([name, 'body', 'thing', '1', '0 0 0', '-', '-', 'frame=%s cushion=%s' % (frame, cushion), 'bench-%d' % seats, '-']))
    # thing.tsv: the old rows of these things go; the new at the foot.
    named = {row.split('\t')[0] for row in rows}
    text = [line for line in lines('thing.tsv') if line != HEAD and line.split('\t')[0] not in named]
    while text and not text[-1].strip():
        text.pop()
    keep('thing.tsv', '\n'.join(text + [HEAD] + rows) + '\n')
    head = "# The couches' backs, arms and bands, from data/once/couch.tsv by script/couch.py."
    kept = [line for line in lines('source.tsv') if line != head and line.split('\t')[0] not in sources]
    while kept and not kept[-1].strip():
        kept.pop()
    keep('source.tsv', '\n'.join(kept + [head] + ['%s\t%s.svg' % (s, s) for s in sources]) + '\n')
    print('%d couches, %d outlines, %d rows' % (len(table['couch']), len(sources), len(rows)))


main()
