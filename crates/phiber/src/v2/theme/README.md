# v2/theme

Theme defaults, resolved style domains, and completed color palettes.

## Files

| File | What it is |
|---|---|
| `descriptor/` | Style descriptor catalog — one row per resolved UI-style field. Each declaration owns one fixed state key, parser, validation bounds, default… |
| `descriptor.rs` | Style descriptor catalog — one row per resolved UI-style field. |
| `mod.rs` | Theme defaults, resolved style domains, and completed color palettes. |
| `palette/` | Color palettes per theme name. Every rendered color resolves through `palette_for(active_theme)`; style geometry resolves through centralized… |
| `resolver.rs` | Pure resolution of canonical UI-style defaults and runtime overrides. |
| `semantic.rs` | Kit-neutral semantic color, gradient, stroke, and effect derivations. |
| `set.rs` | Metric compatibility projection for renderers not yet migrated to `BUiStyleDefaults`. |
| `style/` | Resolved-UI-style types + palette-defaults factory. Palette colors remain the completed color-token source; every other design default is owned… |
| `style.rs` | Resolved-UI-style types + palette-defaults factory. |

Status and next steps: [`jobs.md`](jobs.md).
