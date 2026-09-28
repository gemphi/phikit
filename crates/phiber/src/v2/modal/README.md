# v2/modal

Modal — a top-level overlay peer of Space. Opens on top of the active
space when `ctx.modal` is set. Each modal is a self-contained mini-app.

## Files

| File | What it is |
|---|---|
| `about/` | About window — Blender's `WM_MT_splash_about` (`bl_operators/wm.py:3496`). Left column (65 %): Version, Date, Hash, Branch, Windowing Environment.… |
| `collab/` | Collaboration modal (Activity ▸ Chat / Stream). Shows the session state the collaboration service writes into `BState` — status, server and peers… |
| `command/` | Command Palette — searchable operator/menu launcher. Blender ref: F3 "Menu Search" and "Operator Search" (Developer Extras). |
| `file_browser/` | File Browser modal — the Open / Save As / Import / Export overlay. Content is shared with the `file` host: sidebar categories, columns, filters,… |
| `help/` | Help modal — Blender's Help menu items as an overlay dialog. |
| `item.rs` | `BModalItem` — one modal. |
| `list.rs` | `BModalList` — reads `ctx.modal`; length-0 or length-1. |
| `mod.rs` | Modal — a top-level overlay peer of Space. |
| `profile/` | Profile modal — Phiber user identity + session actions. |
| `reader/` | Reader modal — a document from the declared store (Blender's manual pages, support and community pages, the release notes) read inside the window as a sheet two thirds of the window wide. Every Welcome, Help and About link but the Phiber website opens here; closing it returns to the window it covered. |
| `render/` | Render window — Blender opens `Render ▸ Render Image` (F12) in a new window titled "Blender Render" that hosts an **Image Editor** showing the… |
| `set.rs` | Modal set — one `BModalSpec` per `BModalType`. |
| `settings/` | Settings modal — Preferences dialog opened via `ctx.modal = "settings"`. |
| `spaces/` | Spaces modal — every workspace as a tile; picking one switches `space` and closes the dialog. Driven by `BSpaceType::ALL` (the same set as the… |
| `spec.rs` | `BModalSpec` — static descriptor for one modal. |
| `splash/` | Splash / Welcome window — the Phiber welcome screen. Blender ref: `WM_OT_splash` + `WM_MT_splash` (`bl_operators/wm.py:3431`). Every block is one… |
| `type.rs` | `BModalType` — every overlay dialog that can be opened. |

Status and next steps: [`jobs.md`](jobs.md).
