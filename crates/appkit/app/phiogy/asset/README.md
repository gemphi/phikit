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
- Sixteen scroll couches (`couch-scroll-1` to `-16`), from the user's
  red drawing of a couch's frame (27 November 2026), drawn as lines and
  circles by `script/couch.py` from `data/once/couch.tsv`: a back of
  lobes, one a seat, joined at narrow necks, each a lattice of openings
  (pills or beads) largest at the couch's end; at each end a turned post
  on a ball foot with a spike, and an arm along each side from it to a
  leg in front, flat where a hand rests, rising at the front as a horn
  that ends in a ring (the o). One to four seats, a tall or a low arm, two lattices,
  each its own colours. 32 couches in all.
- Eight lounge chairs and settees of steel bands (`lounge-o-1` ...
  `lounge-u-2`), after the photo of 27 November 2026: each side a bar of
  polished steel bent as O, D, n or u is (`couch-band-*`), white leather
  between; and their small round table with two glasses of water
  (`table-tulip`).
- Sixteen trees of pills, balls, cones and rings, four with letters
  as their trunks (Y, T, S, f) and one a hedge cut as the bird.
- The lionesses Amber and Onyx, Jade the green lion, and six zebras of
  the city's colours, built of pills; a zebra's legs and neck are
  things of their own, stripes and all.
- The gate, ONCE in gold letters 5.5 high, each letter's counter a
  door 1.1 wide; the lotus dome with its lounge; the i of the font as
  the lamps of the way in.
- The floating city's pieces, after the Floating City drawings of
  February 2026 (`D:\Dowloads\OLD-FEB-2026`) and the purple render of
  24 November 2026: the petal dome of gold ribs and glowing lattice,
  cake towers of scalloped tiers under a teal orb, bubble domes, clover
  houses of three-lobed tiers, blob pads with glowing rims, tubes of
  glass for bridges, the wavy ring and the gliders. Scalloped forms
  are the turned forms with lobes (`from` "8 0.08") and the `bloom`
  form, a flat outline of lobes with or without a hole.
- `once-city.usda`, the whole city on its island over the clouds:
  each thing in it is a reference to its own file, so the library's
  scene tree shows the city, the gate, Amber, the lioness and her parts.
- The craft, going +x: eight jetskis (`jetski-1` to `-8`, the two
  hulls `jetski-long` and `jetski-short` raised from the user's red
  sketches, `data/once/hull-*.svg`, in four liveries each, the first
  the white and blue of the photo), eight Phijets, electric
  (`phijet-long-1` to `-4`, `phijet-short-1` to `-4`: the long and the
  short jet of the user's cyan sketch, drawn again as the house's
  letters are built: lines and circles only, every radius a share of
  the nose's, which is the body's height (`data/once/jet.tsv`, drawn
  into `jet-long.svg`, `jet-short.svg` and their windows by
  `script/jet.py`). The nose a quarter circle cut sharp at the keel,
  the tail one upright stroke, the fin swept into the roof through a
  hollow; the cockpit the nose's circle stepped in, the door and
  windows pills. No side wings; small ducted fans low on the rear
  flanks; their bodies in the house's gradients, back to front; the long ones carry
  `jet-inside`, the cabin of club seats down an aisle), four flying cars (`fly-car`, from the
  same sketch, with a glass canopy, leaf windows, hover rings), and
  four rockets as Starship is built (the hopper, the ship, the full
  stack, the ship for Mars with legs and sun wings). Their lights and
  windows are Phiber's o and D.
- Sixteen lands, ice to the tropics (`land-glacier` to
  `land-tropics`), round tiles 40 m across with soft hills one behind
  another as the user's pictures have them, rings of river, peaks
  capped with snow, the city's trees; bright as a theme park's.
  `lands.usda` lays them out four by four.
