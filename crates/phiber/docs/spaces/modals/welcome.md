# Welcome Window - Full Documentation

> **Type:** `BModalType::Splash` (slug `splash`)
> **Size:** 880 × 560, drawn by the full-screen card renderer
> **Source:** `src/v2/modal/splash/{mod,set,recent}.rs`
> **Blender ref:** `WM_MT_splash` — `scripts/startup/bl_operators/wm.py:3431`
> **Status (2026-09-16):** structure and labels at parity; branding is Phiber's.

---

## Layout

```
┌─ banner ──────────────────────────────────── [phiber] Phiber      v0.1.0 ┐
├─ Card: New File ───────────────┬─ Card: Recent Files ────────────────────┤
│  General                       │  scene.blend           ~/Projects       │
│  Shading                       │  … up to 5 rows (RECENT_ROWS)           │
│  UV Editing · Animation …      │  More…            (only when > 5)       │
├─ Card ─────────────────────────┼─ Card ──────────────────────────────────┤
│  Open…                         │  What's New                             │
│  Recover Last Session          │  Support Phiber                         │
└────────────────────────────────┴─────────────────────────────────────────┘
```

With no recent files the right-hand card is **Getting Started** (Manual,
Support, User Communities, Get Involved, Phiber Website), exactly as Blender
swaps `template_recent_files` for its link column.

`BSplashModal::build` returns `Column[ Header, Row[Card, Card], Row[Card, Card] ]`;
`tests/welcome_refinement.rs` asserts that shape.

## Data (`set.rs`, static only)

| Static | Contents | Blender |
|---|---|---|
| `APP_NAME` | `"Phiber"` | splash image text |
| `TEMPLATES` | General, Shading, UV Editing, Animation, Rendering, Compositing, Geometry Nodes, Scripting → space slug | app templates (`TOPBAR_MT_file_new`) |
| `GETTING_STARTED` | 5 links | `WM_MT_splash` L3462–3468 |
| `FOOT_LINKS` | What's New, Support Phiber | L3485–3486 |
| `RECENT_ROWS` / `RECENT_MAX` | 5 / 10 | `template_recent_files(rows=5)` / `FSMENU_RECENT_MAX` |

## Actions

| Control | Action | Effect |
|---|---|---|
| Template row | `Set(ctx.space, <slug>)` | switch workspace, modal closes |
| Recent row | `Set(ctx.file.selected, <path>)` | opens the `.blend` (records it in recents, closes modal) |
| Open… / More… | `Set(ctx.modal, "file_browser")` | File Browser |
| Recover Last Session | `Set(ctx.file.recover_last_session, "true")` | intent key — **reducer not implemented yet** |
| Links | `Set(ctx.link, <url>)` | `handle.rs` calls `cx.open_url` (Blender `wm.url_open`) |

## Gaps

- Recover Last Session has no reducer (needs an autosave / quit.blend writer first).
- Blender's first-run **Quick Setup** page (`WM_MT_splash_quick_setup`) is not ported.
- The splash image itself (Blender draws a release artwork above the columns) is replaced by the banner row.
