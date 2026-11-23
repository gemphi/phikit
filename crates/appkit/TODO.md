# appkit - to do

What is left, the nearest first. What is done is in [`PLAN.md`](PLAN.md)
and, step by step for Phigen, in [`app/phigen/FLOW.md`](app/phigen/FLOW.md).

## Not committed

Nothing of this work is committed. It is staged as one whole: `pui/`,
`src/`, `asset/`, `app/`, `old/`, `build.rs`, the documents, `Cargo.toml`
here and at the workspace's root, `gpu/gpui_glass`, and the files of
`gpu/gpui_windows` that draw glass. The glass crate was taken over on 13
November 2026; its own list is `gpu/gpui_glass/TODO.md`.

## The field

- a selection, with Shift and the arrows; copy and cut
- placing the caret with a press
- text from an input method

## The window

- the caption on the other systems: on macOS and Linux the system's
  own title bar stays over it (on a Mac the three lights would lie on
  the mark); only on Windows does the caption stand in its place. The
  three buttons do not answer the pointer (no hover, no red close),
  maximise does not turn into restore, and a maximised window, Snap
  Layouts and a real hand's drag were not tried: the buttons were
  pressed by message only
- the rest of what the reference shows and Phigen has not:
  `app/phigen/FLOW.md`, "What of the reference is not there"
- the settings: a look that follows the system's light or dark; more to
  choose, such as the size of the type
- the bar's menu: a menu inside a menu (a row that folds its own rows
  open, as puikit's does); a second bar at the right edge for what the
  page shown offers; the row picked and the menu put away kept from
  one opening to the next
- a drawer and a sheet: sliding in and out; a drawer that stays beside
  the page without a veil, so the page is still worked on; a sheet
  dragged up and down by its head
- the models to choose among asked for again while the window is open:
  the server is asked once, the first time the list is wanted
- glass: Reduce Transparency, which makes the glass looks solid; glass
  on the renderers that are not DirectX (`gpu/gpui_glass/TODO.md`); the
  glass looks seen over a busy desktop by a person, not a capture
- the light: a hover and a press bring up the next strength, as puikit's
  controls do; nothing answers the pointer yet
- a picture of a chat longer than the window, and of an answer awaited,
  in today's look; the old ones went to `_temp/phigen`
- the build warns that Phigen's program and its library both write
  `phigen.pdb`; one of the two wants another name
- dragging a finger to scroll
- a modal's content scrolling when it is longer than its panel
- the narrow layout seen on a real phone, in a browser
- a headset: something that shows what `PWindow::place` gives for space

## In a browser

- a call to a server: `PCall` needs the browser's fetch where it has a
  thread today
- a store: `PStore` needs the browser's storage where it has a folder
- a test that runs the browser build

## The kit

- `view`: stack, split, scroll, table, outline, as views of their own
- `widget`: Phigen's parts written in widgets, not drawn by hand
- `service`: command, keymap, session
- gpui's interface half transformed into `pui` (PLAN, step 5)
- one GPU layer, wgpu at a desk too (PLAN, step 6)

## The apps

- Phiace's first window: its tables and its parts; the bar, the tabs and
  the profile come as they are (PLAN, step 7)
- Phiber on `pui`, a piece at a time, once the above holds (PLAN, step 8)

## Asked for, not begun

- `doc/README.md` holds a note asking that documents be made
  automatically, with a `doc` folder to each app and a file to each
  feature. Phigen has `README.md`, `FLOW.md` and `doc/` with its
  pictures; nothing makes them automatically.
