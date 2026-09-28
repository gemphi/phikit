# v2/app

`BPhiberApp` — the V2 GPUI entity. Owns `BState`, builds the tree from
`BSpaceList`, and hands it to `BGpuiRenderer` with an action handler
that calls back into `handle_action` to mutate state.

## Files

| File | What it is |
|---|---|
| `handle.rs` | GPUI action dispatch. |
| `import.rs` | Pure, gpui-free core for import orchestration. |
| `import_tests.rs` | Tests for `should_import` / `settle_import`. |
| `load.rs` | `.blend` open: spawn the phiphx reader off the UI thread, forward its real progress into `scene.load_progress` / `scene.load_stage`, honour the… |
| `load_ui.rs` | "Load UI" (#28) — after a successful open, restore the workspace tab and the 3D view the file was saved with (`WM_OT_open_mainfile.load_ui`). |
| `mod.rs` | `BPhiberApp` — the V2 GPUI entity. |
| `playback.rs` | Playback ticker (Blender `screen.animation_play`): while `K_PLAYING` is true, advance `K_FRAME` by one at the scene's fps (looping through the… |
| `seed.rs` | Seed Phiber's UI state from a loaded file's view layer, so the Outliner and viewport open the way Blender does: the file's active object is the… |
| `settle_tests.rs` | Tests for `settle_import`. |

Status and next steps: [`jobs.md`](jobs.md).
