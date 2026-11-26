"""The Phijets' outlines, built as the house's letters are: straight lines
and circles only.

    python script/jet.py            (from crates/appkit/app/phiogy)

Reads `data/once/jet.tsv`: each outline a chain of circles taken with
the clock, a sharp corner a circle of no radius. Each circle is joined
to the next by the line that touches both, and each is followed round
from where one line leaves it to where the next arrives, so every curve
is a true arc and every join is smooth. Writes the outlines into
`data/once/` (`jet-long.svg`, `jet-long-window-1.svg`, ...), their names
into `source.tsv`, and the jets' body and window rows into `thing.tsv`,
in the place the old ones stood; no other row is touched."""
import math
from fractions import Fraction

from chain import box, g, keep, lines, outline, write


def read():
    """Each jet: its size, and its outlines in order, each a name and
    its circles (centre, signed radius: less than nothing inward)."""
    jet = {}
    for line in lines('jet.tsv'):
        if not line.strip() or line.startswith('#'):
            continue
        thing, piece, kind, cells = line.split('\t')
        value = [float(Fraction(cell)) for cell in cells.split()]
        # A row for every jet: its piece names what it says.
        if thing == '*':
            jet[piece] = value
            continue
        one = jet.setdefault(thing, {'size': None, 'ring': []})
        match kind:
            case '-':
                one['size'] = value
            case 'pill':
                x0, y0, x1, y1 = value
                r = min(x1 - x0, y1 - y0) / 2
                ends = [(x0 + r, y0 + r), (x1 - r, y1 - r)]
                one['ring'].append((piece, [(end, r) for end in ends]))
            case _:
                circle = {'point': lambda x, y: ((x, y), 0.0),
                          'round': lambda x, y, r: ((x, y), r),
                          'hollow': lambda x, y, r: ((x, y), -r),
                          'on': lambda x, y, r, h: ((x + math.sqrt(r * r - (h - y) ** 2), h), 0.0)}[kind](*value)
                if one['ring'] and one['ring'][-1][0] == piece and piece != 'window':
                    one['ring'][-1][1].append(circle)
                else:
                    one['ring'].append((piece, [circle]))
    return jet


def main():
    rows, sources = {}, []
    jet = read()
    # Where a body stands, and how round its edge and its windows' are.
    keel, edge, pane = jet.pop('stand')
    for thing, one in jet.items():
        length, wide = one['size']
        (_, body), *openings = one['ring']
        path, point = outline(body)
        write(thing, path, point, 'The body of %s: lines and circles, from data/once/jet.tsv by script/jet.py.' % thing)
        sources.append((thing, thing))
        x0, y0, x1, y1 = box(point)
        scale = length / (x1 - x0)
        high = (y1 - y0) * scale
        out = ['\t'.join([thing, 'body', 'outline', '0 %s %s' % (g(high), g(wide)), '0 %s 0' % g(keel + high / 2), '-', g(edge), 'body', thing, '-'])]
        for k, (piece, ring) in enumerate(openings):
            name = '%s-window-%d' % (thing, k + 1)
            path, point = outline(ring)
            write(name, path, point, 'The %s of %s, set inside its body: lines and circles.' % (piece, thing))
            sources.append((name, name))
            a, b, c, d = box(point)
            x, y = ((a + c) / 2 - (x0 + x1) / 2) * scale, keel + high / 2 - ((b + d) / 2 - (y0 + y1) / 2) * scale
            for z in (wide / 2, -wide / 2):
                out.append('\t'.join([thing, 'window', 'outline', '%s %s 0.05' % (g((c - a) * scale), g((d - b) * scale)), '%s %s %s' % (g(x), g(y), g(z)), '-', g(pane), 'window', name, '-']))
        rows[thing] = out
    # thing.tsv: the jets' body and window rows in the place of the old
    # ones; their wings, gone.
    text, done = [], set()
    for line in lines('thing.tsv'):
        cell = line.split('\t')
        if cell[0] in rows and len(cell) > 2 and cell[1] in ('body', 'window', 'wing', 'winglet'):
            if cell[0] not in done:
                text += rows[cell[0]]
                done.add(cell[0])
            continue
        text.append(line)
    keep('thing.tsv', '\n'.join(text))
    named = dict(sources)
    head = '# The jets, from data/once/jet.tsv by script/jet.py.'
    # The jets' rows and the line over them go; they are written again
    # at the foot. (The line before them over the old traced jets, too.)
    gone = lambda line: line in (head, '# The jets of the cyan sketch and their windows.') or line.split('\t')[0] in named or line.startswith(tuple(t + '-window-' for t in rows))
    kept = [line for line in lines('source.tsv') if not gone(line)]
    while kept and not kept[-1].strip():
        kept.pop()
    kept += [head] +['%s\t%s.svg' % pair for pair in sources]
    keep('source.tsv', '\n'.join(kept) + '\n')
    print(' '.join('%s %d rows' % (thing, len(out)) for thing, out in rows.items()))


main()
