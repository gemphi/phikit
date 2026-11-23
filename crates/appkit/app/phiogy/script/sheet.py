"""A sheet of the mark's options as a picture (`doc/logo-option.png`):
every option large, then every option at the bar's sizes in each ink
it is tried in. Drawn from the same outlines the mark's file is
written from; a hole is filled with the ground it stands on.

What is tried, at which sizes and in which inks, is `data/logo.tsv`
(`sheet.*`, `try.*`); a colour named there by a token is the theme's
(`data/theme.tsv`), and after `dark/` the dark look's
(`data/look.tsv`)."""
import io
import os

from PIL import Image, ImageDraw, ImageFont


def color(name, here):
    """A colour written out, or the theme's by its token."""
    if name.startswith('#'):
        return name
    look, _, token = name.rpartition('/')
    file, lead = ('look.tsv', [look, 'color', token]) if look else ('theme.tsv', ['color', token])
    for line in io.open(os.path.join(here, 'data', file), encoding='utf-8').read().splitlines():
        cell = line.split('\t')
        if cell[:len(lead)] == lead:
            return cell[len(lead)]
    raise KeyError(name)


def tile(ring, box, side, ink, ground, room, fine):
    """The mark in a square of that side, `room` of it clear all
    round: drawn `fine` times as large and brought down, so its edges
    are smooth."""
    large = side * fine
    scale = large * (1 - 2 * room) / max(box[2] - box[0], box[3] - box[1])
    x, y = (large - (box[2] + box[0]) * scale) / 2, (large - (box[3] + box[1]) * scale) / 2
    picture = Image.new('RGB', (large, large), ground)
    draw = ImageDraw.Draw(picture)
    for number, one in enumerate(ring):
        draw.polygon([(x + px * scale, y + py * scale) for px, py in one], fill=ground if number else ink)
    return picture.resize((side, side), Image.LANCZOS)


def sheet(option, word, here, font, out):
    """`option`: each name with its outlines and their box."""
    side = [int(one) for one in word['sheet.side']]
    gap, room, fine = int(word['sheet.gap'][0]), float(word['sheet.room'][0]), int(word['sheet.fine'][0])
    trial = [(name[4:], color(cell[0], here), color(cell[1], here)) for name, cell in word.items() if name.startswith('try.')]
    paper, ink = color(word['sheet.paper'][0], here), color(word['sheet.ink'][0], here)
    face = ImageFont.truetype(font, int(word['sheet.text'][0]))
    line = int(word['sheet.text'][0]) + gap // 2
    large, small = side[0], side[1:]
    column = large + gap
    strip = sum(one + gap for one in small) * len(trial)
    wide = max(gap + column * len(option), gap * 2 + column + strip)
    high = gap + line + large + gap + line + len(option) * (max(small) + gap)
    picture = Image.new('RGB', (wide, high), paper)
    draw = ImageDraw.Draw(picture)
    # Every option large, in the first ink, its name over it.
    first = trial[0]
    for number, (name, (ring, box)) in enumerate(option.items()):
        left = gap + number * column
        draw.text((left, gap), name, font=face, fill=ink)
        picture.paste(tile(ring, box, large, first[1], first[2], room, fine), (left, gap + line))
    # Every option small, in every ink, a row each.
    top = gap + line + large + gap
    left = gap + column
    for number, (label, _, _) in enumerate(trial):
        draw.text((left + number * sum(one + gap for one in small), top), label, font=face, fill=ink)
    top += line
    for name, (ring, box) in option.items():
        draw.text((gap, top + (max(small) - line) // 2), name, font=face, fill=ink)
        at = left
        for _, mark, ground in trial:
            for one in small:
                picture.paste(tile(ring, box, one, mark, ground, room, fine), (at, top + (max(small) - one) // 2))
                at += one + gap
        top += max(small) + gap
    picture.save(out)
