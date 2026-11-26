"""Outlines built as the house's letters are: a chain of circles taken
with the clock (y runs down), a sharp corner a circle of no radius, a
corner rounded inward a circle of less than no radius. Each circle is
joined to the next by the line that touches both, and each is followed
round from where one line leaves it to where the next arrives, so every
curve is a true arc and every join is smooth. Read by the scripts that
draw Once City's things from their tables (jet.py, couch.py)."""
import io
import math
import os

HERE = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'data', 'once')


def lines(name):
    return io.open(os.path.join(HERE, name), encoding='utf-8').read().split('\n')


def touch(a, b):
    """The line that touches circle `a` and then `b`, the outline outside
    on its left as it is taken with the clock: the angle of its normal,
    pointing out."""
    (ca, ra), (cb, rb) = a, b
    dx, dy = cb[0] - ca[0], cb[1] - ca[1]
    far = math.hypot(dx, dy)
    return math.atan2(dy, dx) - math.acos(max(-1.0, min(1.0, (ra - rb) / far)))


def spot(circle, angle):
    (x, y), r = circle
    return (x + r * math.cos(angle), y + r * math.sin(angle))


def outline(ring, against=False):
    """The circles as one closed path, and its points for its box; taken
    `against` the clock where it is a hole in the outline it is in."""
    count = len(ring)
    normal = [touch(ring[k], ring[(k + 1) % count]) for k in range(count)]
    # Each circle's arc: where it starts and ends, how far it turns.
    arc, point = [], []
    for k, circle in enumerate(ring):
        come, go = normal[k - 1], normal[k]
        span = (go - come) % (2 * math.pi)
        span = span if circle[1] > 0 else span - 2 * math.pi
        arc.append((spot(circle, come), spot(circle, go), circle[1], span))
        point.append(arc[-1][0])
        if abs(circle[1]) > 1e-9:
            point += [spot(circle, come + span * step / 16) for step in range(1, 17)]
    match against:
        # Backwards: each arc from its end to its start, turning the other way.
        case True:
            arc = [(end, start, -r, -span) for start, end, r, span in reversed(arc)]
        case False:
            pass
    path = ''
    for start, end, r, span in arc:
        lead = 'M' if not path else 'L'
        path += '%s%s %s' % (lead, number(start[0]), number(start[1]))
        if abs(r) > 1e-9:
            # Outward it turns with the clock, inward against it.
            big = 1 if abs(span) > math.pi else 0
            path += 'A%s %s 0 %d %d %s %s' % (number(abs(r)), number(abs(r)), big, 1 if span > 0 else 0, number(end[0]), number(end[1]))
    return path + 'Z', point


def number(value):
    return ('%.5f' % value).rstrip('0').rstrip('.')


def g(value):
    return ('%.3f' % value).rstrip('0').rstrip('.')


def box(point):
    xs, ys = [p[0] for p in point], [p[1] for p in point]
    return min(xs), min(ys), max(xs), max(ys)


def write(name, path, point, note):
    """An outline's svg in `data/once/`, its view its points' box; the
    path may hold more than one ring (a hole inside the first)."""
    x0, y0, x1, y1 = box(point)
    text = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="%s %s %s %s">\n  <!-- %s -->\n  <path d="%s"/>\n</svg>\n' % (
        number(x0), number(y0), number(x1 - x0), number(y1 - y0), note, path)
    io.open(os.path.join(HERE, name + '.svg'), 'w', encoding='utf-8', newline='\n').write(text)


def keep(name, text):
    """A table of `data/once/` written again, its lines as given."""
    io.open(os.path.join(HERE, name), 'w', encoding='utf-8', newline='\n').write(text)
