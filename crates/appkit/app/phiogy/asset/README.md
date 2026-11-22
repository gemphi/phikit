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

`icon/phiogy.svg` is Phiogy's mark: the letter o of its own face, a
voice, with a tail leaving its foot at the left, which makes a speech
mark of it. It is the web Phiogy's mark (`puikit/app/phiogy/mark.py`)
on the face this window is set in. `script/logo.py` draws it on the
font's measures, from `data/logo.tsv`; `doc/logo.html` shows it large
and at the bar's size. The row seated `head` in `data/activity.tsv`
names it, and it is drawn in the window's caption, over the bar, in the theme's own
ink, as every icon is.
