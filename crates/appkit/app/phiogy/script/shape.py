"""The outlines of Phiogy's mark as a p with two bars cut out of it, as
the picture the user gave on 22 November 2026 has it: a p in one ink,
its bowl the font's o filled, its stem dropping under it at the left,
and in the bowl a bar and a shorter one beside it, which are holes.

Every outline is a list of pieces, y running down from the bowl's top:
a corner's point, or a round corner as its circle and the angles it
turns through. A picture as a path is written with true curves, so it
is smooth at any size; a picture as dots is drawn from the same pieces
with a point every few degrees (`flat`). The lengths come from `data/logo.tsv` (`p.*`), each a measure
of the font times a number; which corners of the two bars are sharp
comes from `data/logo-corner.tsv`."""
import math


def turn(centre, radius, start, end):
    """A round corner: part of a circle, from one angle to another in
    degrees. With no radius it is its one point."""
    return ('turn', centre, radius, start, end) if radius > 0 else ('point', centre)


def at(centre, radius, angle):
    return (centre[0] + radius * math.cos(math.radians(angle)), centre[1] + radius * math.sin(math.radians(angle)))


def flat(ring, fine):
    """An outline as points alone: every round corner as a point every
    `fine` degrees or so."""
    out = []
    for piece in ring:
        if piece[0] == 'point':
            out.append(piece[1])
            continue
        _, centre, radius, start, end = piece
        count = max(1, int(abs(end - start) / fine))
        out += [at(centre, radius, start + (end - start) * step / count) for step in range(count + 1)]
    return out


def bar(box, radius, corner):
    """A box (left, top, right, bottom) whose corners, from the top
    left round with the clock, are each round (`r`) or sharp (`s`)."""
    left, top, right, bottom = box
    seat = [((left, top), (left + radius, top + radius)), ((right, top), (right - radius, top + radius)), ((right, bottom), (right - radius, bottom - radius)), ((left, bottom), (left + radius, bottom - radius))]
    out = []
    for number, (letter, (point, centre)) in enumerate(zip(corner, seat)):
        start = 180 + number * 90
        out.append(turn(centre, radius, start, start + 90) if letter == 'r' else ('point', point))
    return out


def body(size):
    """The p, filled: its bowl round at the right, more at the top
    than at the foot; a round hollow where the bowl's foot meets the
    stem; the stem's foot as round at its right as the table says; and
    its two left corners round, at the top (`left`) and at the stem's
    foot (`heel`)."""
    wide, high, round, low = size['p.wide'], size['p.high'], size['p.corner'], size['p.corner.low']
    stem, drop, hollow, foot = size['p.tail.wide'], size['p.tail.drop'], size['p.hollow'], size['p.foot']
    left, heel = size['p.left'], size['p.heel']
    return [
        turn((left, left), left, 180, 270),
        turn((wide - round, round), round, 270, 360),
        turn((wide - low, high - low), low, 0, 90),
        turn((stem + hollow, high + hollow), hollow, 270, 180),
        turn((stem - foot, high + drop - foot), foot, 0, 90),
        turn((heel, high + drop - heel), heel, 90, 180),
    ]


def mark(size, corner):
    """The mark's outlines: the p, then the two holes in it, whose
    corners are as `corner` says for the bar and for the short one. And
    the box round them all."""
    left = size['p.wall']
    long = (left, size['p.top'], left + size['p.bar.wide'], size['p.top'] + size['p.bar.high'])
    start = long[2] + size['p.gap']
    short = (start, size['p.dot.top'], start + size['p.dot.wide'], size['p.dot.top'] + size['p.dot.high'])
    ring = [body(size), bar(long, size['p.bar.corner'], corner[0]), bar(short, size['p.dot.corner'], corner[1])]
    return ring, (0, 0, size['p.wide'], size['p.high'] + size['p.tail.drop'])


def back(ring):
    """An outline the other way round."""
    return [piece if piece[0] == 'point' else ('turn', piece[1], piece[2], piece[4], piece[3]) for piece in reversed(ring)]


def path(ring, number, place=(1, 0, 0), wound=False):
    """Outlines as one path, every round corner a true arc: it turns
    with the clock where its angles grow. `place` is how large it is
    written and where its top left is. `wound` writes the holes the
    other way round than the p, so they are holes also where a picture
    is filled without the rule that says so."""
    scale, left, top = place
    spot = lambda point: '%s %s' % (number(left + point[0] * scale), number(top + point[1] * scale))
    out = ''
    for index, one in enumerate(ring):
        lead = 'M'
        for piece in back(one) if wound and index else one:
            if piece[0] == 'point':
                out += lead + spot(piece[1])
            else:
                _, centre, radius, start, end = piece
                size = number(radius * scale)
                out += '%s%sA%s %s 0 0 %s %s' % (lead, spot(at(centre, radius, start)), size, size, 1 if end > start else 0, spot(at(centre, radius, end)))
            lead = 'L'
        out += 'Z'
    return out
