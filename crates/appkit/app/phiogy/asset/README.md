# Phiogy's assets

Files only Phiogy has. A file here is found before one of the same path
in `crates/appkit/asset`, which every app shares.

Its icons (`chat`, `globe`, `layout-grid`, `home`, `user`, `mic`, `eye`,
`grid`, `volume`, `box`, `check-circle`) and its typeface are the shared
ones. `data/icon.tsv` says which shared icon each of the web Phiogy's
icons is drawn with. Its own are its mark, three of the viewer's
tools, and the icons of a room's controls and marks (`reply`,
`heart`, `pin`, `pin-off`, `send`, `star`, `message-circle`, `users`,
`hash`, `thumbs-up`, `flame`, `search`, `check`, `user-plus`, `plus`,
`shopping-bag`, `home`, `layers`, `wallet`, `log-in`, `log-out`,
`trash-2`, `bell-off`, `image-plus`): lucide's drawings, as the web
draws them, written by `node script/icon.mjs NAME...` from the copy of
lucide in `puikit/node_modules/lucide-react`.

## The viewer's tools

`icon/share-2.svg`, `icon/download.svg` and `icon/external-link.svg`
are lucide's drawings of those names, which the web's viewer uses
(`puikit/src/components/overlays/Lightbox`), written here from the
copy of lucide in `puikit/node_modules/lucide-react`, in the form of
the shared icons and under the same licence
(`crates/appkit/asset/icon/LICENSE`). The shared icons have no share,
download or open-outside; `zoom-in`, `zoom-out`, `x` and the arrows
are the shared ones. `data/tool.tsv` says which tool has which.

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
