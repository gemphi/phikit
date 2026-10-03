# phiui — theming (vendored)

phiber draws its own widgets (`src/v2/view`, and the older `src/bui`), so the
vendored widget set is gone: `ui`, `ui_macros`, `component`, `icons` and `menu`
were removed from the workspace once nothing referenced them (the one call left,
a tooltip, is now `bui::core::BTip`).

| Crate | Used by | What it does |
|---|---|---|
| `theme` | phiber (`app/mod.rs`: `theme::init`, `LoadThemes`, `ThemeSettingsProvider`) | theme settings and the loaded colour set |
| `syntax_theme` | `theme` | the syntax colours a theme carries |

Check before adding anything here: `cargo tree -p phiber` is the test of whether
a crate is really needed.
