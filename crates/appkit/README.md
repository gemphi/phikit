# appkit

The common layer every phi app stands on. An app builds no window of its
own: it hands appkit tables (its spaces, hosts, parts, the rows of its
bar, its user, its look, its words) and the view of each of its parts,
and appkit opens the window, lays it out for a desk or a hand, draws it,
and sends it what a hand does. The same app runs in a browser.

Phigen, a chat, is the first app on it: [`app/phigen`](app/phigen/README.md).

```bash
cargo run -p phigen
```

```bash
cargo test -p appkit -p phigen
```

## What is where

| Folder | Holds |
|---|---|
| [`pui/`](pui/mod.rs) | the UI kit: what a thing on screen is (`core`, `control`, `layout`, `widget`) and how a window is built (`window`); `app.rs` is what an app supplies |
| [`src/`](src/lib.rs) | what is not the kit: `theme`, `service`, `screen` (the one place that names gpui), `data`, and the tests in `test` |
| [`asset/`](asset) | the icons and the typeface every app shares; `build.rs` builds them into the program |
| [`app/`](app) | the apps, each with its own `data/`, `asset/` and documents |
| [`gpu/`](gpu) | Zed's gpui: the system window, the renderers, the text; and `gpui_glass` |
| [`old/`](old) | the earlier AppKit-style framework, kept as reference; not built |

## Making an app

1. Tables in `data/`: `space`, `layout`, `host`, `region`, `part`,
   `modal`, `activity`, `user`, `theme`, `text`. Phigen's are the model.
2. A type that is [`PApp`](pui/app.rs): it reads the tables and gives the
   view of each part. A view is [`PView`](pui/core/view.rs): it draws on
   a [`PPaint`](pui/core/paint.rs) and answers a [`PEvent`](pui/core/event.rs).
3. `app.stage()` and [`screen::run`](src/screen/mod.rs).

The activity bar, the user's row, the tabs, the profile and a modal's
panel are appkit's and need no code from the app. Every colour, size and
face is a token of the app's `theme.tsv`.

## Documents

| File | Says |
|---|---|
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | the diagrams: the layers, a frame, an event, an answer over time |
| [`PLAN.md`](PLAN.md) | what was decided, the hierarchy, who owns what, the steps |
| [`SPECS.md`](SPECS.md) | each piece: what it must do, Apple's name for it, what was read, the test |
| [`TODO.md`](TODO.md) | what is left, in order |
| [`app/phigen/FLOW.md`](app/phigen/FLOW.md) | the steps a person takes in Phigen, each a passing test, and its flow charts |

## Rules

Every type starts with `P`. Every name is one word and singular. A thing
of one file is a file. No file passes 250 lines. `match`, not `else if`.
Nothing a person reads and no colour or size is in code. The tests are
apart from the code, in the same folders, under `src/test`.
