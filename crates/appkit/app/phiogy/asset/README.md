# Phiogy's assets

Files only Phiogy has. A file here is found before one of the same path
in `crates/appkit/asset`, which every app shares.

Its icons and its typeface are the shared ones. Every shared icon is
the house's own, drawn from measures: `crates/appkit/asset/icon*.tsv`
holds each icon's points and how round each corner is, and
`crates/appkit/script/icon.py` draws the svg files from it. Nothing
here is lucide's any more; the copies that were are in
`gemphi/_temp/phiogy-lucide-icons-2026-11-23`, with the script that
made them. `data/icon.tsv` is a map of names: which of the house's
icons each of the web Phiogy's names is drawn with. `data/tool.tsv`
says which of the viewer's tools has which icon.

Its own files here are its mark and the two bars cut out of it.

## The logo

`icon/phiogy.svg` is Phiogy's mark: a p in one ink, its bowl the box
of the letter o of its own face, filled, with two bars cut out of it,
a long one, round at its top left and its bottom right and sharp at
the other two corners, and a shorter one round all over. It is the
picture the user gave on 22 November 2026, traced, its pink parts
holes: it ends where the picture ends, the hollow under the bowl
coming down to a point at the stem's foot, and its top left and
bottom left corners are round, fitted to the screenshot the user gave
of the look they chose. `script/logo.py` draws it on the font's measures, from
`data/logo.tsv`, and `data/logo-corner.tsv` holds the options of which
corners of the bars are sharp; `doc/logo-option.png` shows every
option in every ink tried, `doc/logo.html` the chosen one large and
at the bar's size, and `doc/logo/` keeps every drawing. The mark
before it, the letter o with a tail (the web Phiogy's,
`puikit/app/phiogy/mark.py`), is `doc/logo/voice.svg`, and the row
`mark voice` of `data/logo.tsv` puts it back. The web's kit keeps the
same p as the `icon` of Phiogy's entry in
`puikit/app/phi-kit/src/registry.json`: the script writes it as
`doc/logo/registry-icon.json`, and it is put there from that file. The row seated `head` in `data/activity.tsv`
names it, and it is drawn in the window's caption, over the bar, in the theme's own
ink, as every icon is.

## Once City

`once/` holds Once City's things as USD, a file each, written by
phiphx's `thing_usd` example from the tables in `data/once/`
(`thing.tsv` the parts, `look.tsv` the looks, `source.tsv` the font and
the drawing they are made from); a test there keeps the files equal to
the tables. Nothing in a file is drawn by hand.

- Sixteen couches, each a letter of Phi-Pill Bold as its two side
  panels with seat and back cushions between (L, h, n, u, d, b, r, e,
  c, z, J, w), the I lying as an ottoman, the o as a pouf, the m as a
  stool, and the bird (the p above) as the rose chaise's panels. The
  seats are 0.46 high.
- Sixteen trees of pills, balls, cones and rings, four with letters
  as their trunks (Y, T, S, f) and one a hedge cut as the bird.
- The lionesses Amber and Onyx, Jade the green lion, and six zebras of
  the city's colours, built of pills; a zebra's legs and neck are
  things of their own, stripes and all.
- The gate, ONCE in gold letters 5.5 high, each letter's counter a
  door 1.1 wide; the lotus dome with its lounge; the i of the font as
  the lamps of the way in.
- `once-city.usda`, the whole city on its island over the clouds:
  each thing in it is a reference to its own file, so the library's
  scene tree shows the city, the gate, Amber, the lioness and her parts.
