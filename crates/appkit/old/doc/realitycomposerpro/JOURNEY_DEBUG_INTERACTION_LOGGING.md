# Journey: Interaction Logging + Debug Trace (Rust Kit Pro)

This journey lets you **turn on structured logs**, interact with the editor, and then use the output to pinpoint what needs fixing.

## References (Docs)

- Reality Composer Pro overview:
  - `app/phipro/doc/read/realitycomposerpro/-realitycomposerpro.md`
- Configuring the project window:
  - `app/phipro/doc/read/realitycomposerpro/configure-the-reality-composer-pro-project-window/--configure-the-reality-composer-pro-project-window.md`
- Project Browser expectations:
  - `app/phipro/doc/read/realitycomposerpro/editor_projectbrowser/--editor_projectbrowser.md`

## What You Enable

### Environment Variables

Set these before you run the app:

- `RUST_APPKIT_TRACE_HITS=1`
  - Logs which view receives clicks (hit-testing + dispatch tracing)
- `RUST_KIT_PRO_TRACE_ACTIONS=1`
  - Logs every `menu::Action` routed through `ActionHandler::handle`
- `RUST_KIT_PRO_TRACE_IMPORT=1`
  - Logs per-file import start/ok/error and destination output path

## How to Run

From the workspace root (`phi/`):

- `cargo run -p rust-kit-pro`

If Windows reports "file in use" during rebuild, close the running `rust-kit-pro.exe` and retry.

## Step-by-Step Interaction Script

### Step 1 — Confirm click routing (View hit logs)

Actions:

1. Click the **Hierarchy** panel.
2. Click the **Viewport** panel.
3. Click the **Inspector** panel.
4. Click inside **Bottom Tabs** and on the tab labels.

Expected logs:

- `[Dispatch] View: '<name>' ...`
- `[Mouse] Click hit view: '<name>' ...`

If logs show clicks landing on the wrong view, the bug is in:

- view frames / autoresizing
- or hit-testing (event coordinate transforms)

### Step 2 — Confirm tab switching actions

Actions:

1. Use the top header mode switch (Scene/Shaders/Timeline) if present.
2. Click bottom tabs: `Timeline`, `Project`, `Assets`, `Shader Graph`.

Expected logs:

- `[Action] SetScreenMode(<n>)` for workspace mode switching

If tab clicks do not switch views:

- verify the click hits `BottomTabs` (Step 1)
- verify the delegate is installed on `NSTabView`

### Step 3 — Confirm theme switching applies to all panels

Actions:

1. Trigger a theme switch from the menu (or existing theme UI).

Expected logs:

- `[Action] SetTheme("...")`

Visual expectation:

- Viewport background, panels, and toolbars should repaint

If some panels do not repaint:

- verify their view names match theme-token assignment patterns
- ensure they have `theme_token` set via `setup_theme_tokens`

### Step 4 — Asset import (menu + Project Browser button)

Actions:

1. Use `File > Import`.
2. Also click `Project Browser > Import` button.

Suggested test files:

- USDA file (should work without Blender):
  - `assets/botanist/usd/environment/mesh/volume.usd`
- USDZ files (may require Blender fallback):
  - `assets/botanist/usd/Robot/mesh/body1.usdz`

Expected logs:

- `[Action] FileImport`
- `[Import] start src='...' dst='.../assets/imported/<name>.phx'`
- `[Import] ok dst='...'` OR `[Import] err src='...' reason='...'`

If you see Blender-related failure:

- install Blender and ensure `blender` is on PATH

If you see USD parse failure:

- confirm whether the USDZ payload is binary USDC (OpenUSD required)

## What to Paste Back Here

After you run through the steps, paste:

- The console block from the first click you do
- The console block from one full import attempt
- The action log around theme switching

Then I’ll tell you exactly where the mismatch is and what file to fix.
