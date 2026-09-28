# File Browser - Full Documentation

> **Types:** `BModalType::FileBrowser` (dialog) · `BHostType::File` / `BHostType::Asset` (editors)
> **Source:** `src/v2/part/custom/file/*` (shared) · `src/v2/modal/file_browser/*` (re-exports)
> **Blender ref:** `scripts/startup/bl_ui/space_filebrowser.py`, `editors/space_file/space_file.cc`
> **Status (2026-09-16):** one implementation for dialog and editors; headings, sections and menus at Blender parity.

---

## One implementation, three places

Before this pass the modal and the editors had two different sidebars (different
headings, keys and accordion styles). Now every piece lives once in
`part/custom/file/` and the modal only re-exports it:

| Blender region | Module | Dialog | File editor | Asset editor |
|---|---|---|---|---|
| HEADER (`FILEBROWSER_MT_editor_menus`) | `header.rs` | drawn by the modal | host Header region (`menubar/set.rs` `FILE_HOST`) | host Header (`ASSET`) |
| UI (`FILEBROWSER_PT_directory_path`) | `path_bar.rs` | ✓ | ✓ | ✓ |
| TOOLS (`FILEBROWSER_PT_bookmarks_*`) | `sidebar.rs` / `asset_sidebar.rs` | ✓ | ✓ | library + catalogs |
| WINDOW (file list) | `table.rs` | ✓ | ✓ | ✓ |
| EXECUTE (`file_panel_execution_buttons_draw`) | `execute.rs` | ✓ (Cancel closes modal) | only while a purpose is set (Cancel clears it) | — |

`BFileItem` (editors) and `BFileBrowserModal` assemble them in that order.
`tests/host_visual_fixes.rs::modal_file_browser_shares_the_host_sidebar` asserts
the two sidebars are the identical tree.

## Tools region — Bookmarks panels

Blender `bl_region_type = 'TOOLS'`, `bl_category = "Bookmarks"`, in draw order:

| Panel | Blender class | Rows |
|---|---|---|
| Volumes | `FILEBROWSER_PT_bookmarks_volumes` (L221) | mounted drives (`fsmenu::volumes`) |
| System | `FILEBROWSER_PT_bookmarks_system` (L244) | Home, Desktop, Documents… that exist |
| Bookmarks | `FILEBROWSER_PT_bookmarks_favorites` (L282) | `ctx.file.bookmark.<0..9>` + **Add Bookmark** (`file.bookmark_add`) |
| Recent | `FILEBROWSER_PT_bookmarks_recents` (L332) | shared with the welcome window's recent list |

Each panel is a rounded `Card` whose first child is the same `AccordionHeader`
the Properties editor uses; expansion key `ctx.file.sidebar.<label>.expanded`
(open by default). The panel table is `set.rs::CATEGORIES`.

Asset mode (`asset_sidebar.rs`): **Library** (Current File / Essentials / All →
`ctx.asset.library`) and **Catalogs** (All / Unassigned → `ctx.asset.catalog`).

## Header menus (Blender items, Blender order)

| Mode | Menu | Items |
|---|---|---|
| Files | View | Source List, File Path, Frame Selected · Display Size, Recursion Level · Area |
| Files / Assets | Select | All, None, Invert · Box Select |
| Assets | View | Source List, Asset Details, Frame Selected · Display Size · Area |
| Assets | Library | Refresh, Refresh Remote Listing |
| Assets | Catalog | Undo, Redo · Save Catalogs, New Catalog |

## Path bar

Back · Forward · Parent · Refresh │ directory │ search │ display modes (list /
grid / thumbnails) │ show-hidden eye │ filter toggle │ **filter popover**
(`FILEBROWSER_PT_filter`: Show Hidden + the per-type flags from `set.rs::FILTERS`).

## Purposes

`ctx.file.purpose` (`purpose.rs`): `open`, `save_as`, `import`, `export`,
`link`, `append`. Menu items declared with `op_file(label, shortcut, purpose)`
set it; the reducer (`state/action.rs`) raises the modal when no modal is open.
The execute button label follows the purpose; button order follows the
platform (affirmative first on Windows).

## Gaps

- Display Size / Recursion Level / Area submenus are empty; Source List / File Path toggles don't yet hide regions.
- Bookmark removal (`file.bookmark_delete`) and reordering are not implemented.
- ✅ `.blend` thumbnails (Blender's `TEST` block) + `REND` subtitle show in the Thumbnails display (`thumbs.rs`, `preview.rs`).
- Horizontal List display still renders as the vertical list.
- ✅ Operator options panel (`options.rs`): **Load UI** for Open, after the four Bookmarks panels.
- Asset catalogs are not read from `blender_assets.cats.txt`; only All / Unassigned exist.
