# Phigen's assets

Files only Phigen has. A file here is found before one of the same path
in `crates/appkit/asset`, which every app shares.

Its icons (`home`, `chat`, `layout-grid`, `sparkles`, `arrow-up`) and
its typeface are the shared ones. Its own is its mark.

## The logo

`icon/phigen-chat.svg` and `icon/phigen-bar.svg` are Phigen's mark, two
ways: the font's o as a chat bubble with a beak under its left corner
and two lines in it, and two lying bars that make a P. In both the
upper line is twice the lower. `script/logo.py` draws them on the
measures of Phigen's own face, from `data/logo.tsv`; `doc/logo.html`
shows both. The one named in the icon cell of the row seated `head` in
`data/activity.tsv` is drawn in the window's caption, over the bar, in the theme's own
ink, as every icon is.
