# Phiber — Modal UI Reference

> **Location**: `crates/phiber/docs/modals.md`
> **v2 (current):** specs live in `src/v2/modal/set.rs`, one content module per
> modal under `src/v2/modal/<name>/`. Sections 1–3 below describe the legacy
> `store/modals/*` data those v2 modules were ported from; sections 7–10 cover
> the modals added in v2. Last updated 2026-09-16.

---

## Overview — Eight Modals (v2)

| `BModalType` | Title | Size (w×h) | Content module | Blender counterpart |
|---|---|---|---|---|
| `Settings` | Preferences | 960×640 | `modal/settings` | `SpacePreferences` (Edit ▸ Preferences) |
| `Help` | Help | 900×640 | `modal/help` | Help menu / manual |
| `Profile` | Profile | 480×520 | `modal/profile` | — (Phiber account) |
| `Command` | Command Palette | 640×420 | `modal/command` | Menu Search (F3) |
| `Splash` | Welcome to Phiber | 880×560 (full-screen card) | `modal/splash` | `WM_MT_splash` |
| `About` | About Phiber | 640×480 (full-screen card) | `modal/about` | `WM_MT_splash_about` |
| `FileBrowser` | Phiber File View | 1024×640 | `modal/file_browser` → `part/custom/file` | `SpaceFile` in a temp window |
| `Render` | Phiber Render | 1100×720 | `modal/render` (Image host) | "Blender Render" window |

Modals sit above the full app layout as GPUI overlay panels, centered on screen.
`ctx.modal = <slug>` opens one; any committed `Set` inside it (or the backdrop /
title-bar close) dismisses it. `BModalItem` wraps each body in `BView::Modal`;
`Splash` and `About` render through `view/render/gpui/modal/fullscreen.rs`,
the rest through the card renderer (`modal/card.rs`).

---

## 1. SETTINGS MODAL (BModalType::Settings — 640px wide)

```
+--------------------------------------------------------------------------+
| (1) Modal Chrome (role: "modal", h: auto, w: 640px, rounded: MD)        |
| +------------------------------------------------------------------------+|
| | (2) Title Bar (role: "header", h: 32px, flex-row)                    ||
| |  [settings icon]  "Preferences"                              [x]      ||
| |   BView::label_icon()              text              BView::IconButton ||
| +------------------------------------------------------------------------+|
| +----------+ +----------------------------------------------------------+ |
| | (3) Tabs | | (4) Content Panel (role: "panel", flex-col, flex-1)     | |
| | w: 160px | |                                                          | |
| | flex-col | |  (5) BView::Section("Interface", icon, expanded, [...]) | |
| |          | |  +----------------------------------------------------+  | |
| | (6) Tab  | |  | (7) BView::Row(gap:None, [label, value])           |  | |
| |>Interface| |  |   "Display"                 "Default"              |  | |
| | Editing  | |  |   "Resolution"              "Auto"                 |  | |
| | Navigate | |  |   "Scale"                   "1.0x"                 |  | |
| | Add-ons  | |  |   "Language"                "English"              |  | |
| | Themes   | |  +----------------------------------------------------+  | |
| | System   | |                                                          | |
| | ---      | +----------------------------------------------------------+ |
| | Keymap   |                                                              |
| | Extens.  |                                                              |
| +----------+                                                              |
+--------------------------------------------------------------------------+
```

### Settings Tabs and Rows (from store/modals/settings/def.rs)

```
Tab: Interface   rows() -> INTERFACE_ROWS
  Display        -> Default
  Resolution     -> Auto
  Scale          -> 1.0x
  Language       -> English

Tab: Editing     rows() -> EDITING_ROWS
  Auto-Save      -> Enabled
  Undo Steps     -> 32
  Snap           -> Grid

Tab: Navigation  rows() -> NAVIGATION_ROWS
  Orbit          -> Turntable
  Zoom           -> Scale
  Walk           -> Disabled

Tab: Add-ons     rows() -> ADDONS_ROWS
  Installed      -> 0
  Check Updates  -> Manual

Tab: Themes      [EXPANDED - see settings.md]
  Active Theme   -> 8 theme cards picker
  Accent Color   -> 16 swatch picker
  BG Effect      -> Fill / Gradient toggle
  Corner Radius  -> XS SM MD LG XL
  UI Density     -> XS SM MD LG XL
  Font Family    -> dropdown

Tab: System      rows() -> SYSTEM_ROWS
  GPU            -> Auto
  Memory         -> Default
  Sound          -> Enabled
```

### Zed Settings Keys Mapped to Phiber Tabs

```
Interface tab:
  "ui_font_family"          string
  "ui_font_size"            number (10-24)
  "buffer_font_family"      string
  "buffer_font_size"        number (10-24)
  "line_height"             { "custom": 1.5 }

Theme tab:
  "theme"                   "Phisia Dark"
  "icon_theme"              "default"
  "ui_density"              compact | default | comfortable

Editing tab:
  "autosave"                off | after_delay | on_focus_change
  "undo_depth"              number (1-1000)
  "tab_size"                number (1-8)
  "hard_tabs"               bool
  "format_on_save"          on | off
  "remove_trailing_whitespace_on_save"  bool
  "ensure_final_newline_on_save"        bool
  "show_whitespace"         all | selection | none

Navigation tab:
  "vertical_scroll_margin"  number
  "scroll_sensitivity"      number
  "vim_mode"                bool
  "relative_line_numbers"   bool

System tab:
  "gpu_acceleration"        on | off
  "telemetry.diagnostics"   bool
  "telemetry.metrics"       bool
```

---

## 2. HELP MODAL (BModalType::Help — 480px wide)

```
+----------------------------------------------------+
| (1) Modal Chrome (role: "modal", w: 480px)         |
| +--------------------------------------------------+|
| | (2) Title Bar (role: "header", h:32px)           ||
| |  [help icon]  "Help & Documentation"         [x] ||
| +--------------------------------------------------+|
| +-----------+ +-----------------------------------+ |
| | (3) Nav   | | (4) Doc Content (flex-col, flex-1)| |
| | w: 160px  | |                                   | |
| | BView::   | |  (5) BView::label_icon(           | |
| | VTabBar   | |        label, icon)                | |
| |           | |  ---------------------------      | |
| | [book]    | |  (6) BView::Scroll([              | |
| |  Docs     | |    BView::label(doc_text)          | |
| | [play]    | |  ])                                | |
| |  Tutorials| |                                   | |
| | [keyboard]| |                                   | |
| |  Keys     | |                                   | |
| | [code]    | |                                   | |
| |  API      | |                                   | |
| | [users]   | |                                   | |
| |  Forum    | |                                   | |
| | [bug]     | |                                   | |
| |  Bug      | +-----------------------------------+ |
| +-----------+                                       |
+----------------------------------------------------+
```

### Help Items (from store/modals/help/def.rs)

```
BHelpType::Documentation   icon:"book"
  -> "Phiber is a next-generation 3D editor built on top of GPUI..."

BHelpType::Tutorials       icon:"play"
  -> "1. Open the View3D space... 2. Use the Outliner..."

BHelpType::KeyboardShortcuts icon:"keyboard"
  -> "Space: play/pause  G: grab  R: rotate  S: scale  Tab: mode"

BHelpType::ApiReference    icon:"code"
  -> "Store crates define static specs. State crates track runtime..."

BHelpType::CommunityForum  icon:"users"
  -> "Discord: coming soon  Forum: placeholder  GitHub: private beta"

BHelpType::ReportBug       icon:"bug"
  -> "Please include: version, steps to reproduce, expected/actual..."
```

---

## 3. PROFILE / ACCOUNT MODAL (BModalType::Profile — 400px → 440px target)

### Current State

```
+--------------------------------------+
| (1) Modal Chrome (w:400)             |
| +------------------------------------+|
| | (2) Title: "Account"              ||
| +------------------------------------+|
| (3) BView::Column([                  |
|   BView::label_icon("Sign In",       |
|                     "login")         |
|   BView::label_icon("Preferences",   |
|                     "settings")      |
|   BView::label_icon("Theme Settings",|
|                     "palette")       |
|   BView::label_icon("Keyboard...",   |
|                     "keyboard")      |
|   BView::Separator()                 |
|   BView::label_icon("Sign Out",      |
|                     "logout")        |
| ])                                   |
+--------------------------------------+
```

### Target State — Full Discord-style Account Modal

```
+----------------------------------------------------------+
| (1) Modal Chrome (role:"modal", w:440px)                 |
|                                                          |
| +--------------------------------------------------------+|
| | (2) Profile Banner (h:80px, accent gradient bg)       ||
| |  +----+  Phiace Studio                    [Edit]      ||
| |  | A  |  @phiace                                      ||
| |  +----+  (.) Online                                   ||
| |  avatar  status dot                                   ||
| +--------------------------------------------------------+|
|                                                          |
| +--------------------------------------------------------+|
| | (3) Status Picker (BView::HTabBar, flex-row)           ||
| |  [(.) Online]  [(~) Idle]  [(x) DND]  [( ) Invisible]||
| +--------------------------------------------------------+|
|                                                          |
| +--------------------------------------------------------+|
| | (4) Custom Status (BView::Row, flex-row)              ||
| |   [emoji]  "What is your status?"                     ||
| +--------------------------------------------------------+|
|                                                          |
| -------------------------------------------------------- |
|                                                          |
| (5) BView::Scroll([                                      |
|   (6) BView::label("Recent Activity")  ← feed header    |
|   --------------------------------------------------     |
|   (7) Feed Items (BView::Row each):                      |
|     [saved-icon] "Saved diorama.phib"        2m ago     |
|     [view-icon]  "Opened 3D Viewport"        5m ago     |
|     [theme-icon] "Changed theme -> Ocean"    12m ago    |
|     [addon-icon] "Installed add-on Phiray"   1h ago     |
|     [user-icon]  "Joined Phiber Beta"        2d ago     |
| ])                                                       |
|                                                          |
| -------------------------------------------------------- |
|                                                          |
| (8) Action Links (BView::Column):                        |
|   [settings] Preferences                                 |
|   [palette]  Theme Settings                              |
|   [keyboard] Keyboard Shortcuts                          |
|   [arrow]    View Profile                                |
|   ----------                                             |
|   [logout]   Sign Out                                    |
+----------------------------------------------------------+
```

### New Store Data Needed

```rust
// store/modals/profile/def.rs — NEW additions

pub enum BStatusType {
    Online,    // green dot
    Idle,      // amber dot
    Dnd,       // red dot (Do Not Disturb)
    Invisible, // grey dot
}

pub struct BActivityEntry {
    pub icon: &'static str,      // SVG asset key
    pub text: &'static str,      // "Saved diorama.phib"
    pub timestamp: &'static str, // "2m ago"
}

pub static ACTIVITY_FEED: &[BActivityEntry] = &[
    BActivityEntry { icon:"save", text:"Saved diorama.phib", timestamp:"2m ago" },
    BActivityEntry { icon:"view3d", text:"Opened 3D Viewport", timestamp:"5m ago" },
    BActivityEntry { icon:"palette", text:"Changed theme -> Ocean", timestamp:"12m ago" },
    BActivityEntry { icon:"box", text:"Installed add-on Phiray", timestamp:"1h ago" },
    BActivityEntry { icon:"user", text:"Joined Phiber Beta", timestamp:"2d ago" },
];
```

---

## 4. ELEMENT COMPONENT MAP

All modals use these BView primitives:

```
BView::Panel(role, title, children)
  "modal"   -> full modal chrome, overlay shadow, rounded corners
  "header"  -> title bar, h:32px, flex-row, icons + text
  "panel"   -> content area, flex-col, flex-1, p:8px
  "banner"  -> [NEW] account header banner, h:80px, accent bg

BView::VTabBar(children)          -> vertical nav sidebar (Settings, Help)
BView::Tab(label, active)         -> single nav tab item
BView::tab_icon(label, icon, active) -> nav tab with SVG icon

BView::Section(label, icon, expanded, children)
  -> collapsible section header (used in Settings content panel)

BView::Row(gap, [label, value])
  -> key-value setting row OR horizontal layout item

BView::Scroll(children)
  -> scrollable doc body / activity feed container

BView::label(text)                -> plain text
BView::label_icon(text, icon)     -> icon + text (menu items, action links)
BView::separator()                -> horizontal divider line
BView::IconButton(icon)           -> close [x] button, [Edit] button
```

---

## 5. MODAL TRIGGER POINTS

Menu items declare their target in `part/menubar/set.rs` — `op_modal` (opens a
modal), `op_file` (opens the File Browser **with a purpose**: open, save_as,
import, export, link, append) and `op_url` (system browser via `ctx.link`,
Blender's `wm.url_open`). `leaf_action` routes them; no per-label tables.

```
Welcome (Splash)
  <- shown once at startup (state seed writes ctx.modal = "splash")
  <- Topbar "Phiber" menu -> "Splash Screen"

About
  <- Topbar "Phiber" menu -> "About Phiber"

File Browser
  <- File -> Open... / Save As... / Save Copy... / Link... / Append...
  <- File -> Import > * / Export > * / Export All Collections
  <- Render -> Render Audio... ; Window -> Save Screenshot...
  <- Welcome -> Open... / More...
  (setting ctx.file.purpose raises the modal when none is open)

Render
  <- Render -> Render Image (F12) / Render Animation (Ctrl F12)
  <- Render -> View Render (F11) / View Animation (Ctrl F11)

Command Palette
  <- Edit -> Menu Search... (F3)

Settings Modal
  <- Topbar "Edit" menu -> "Preferences"
  <- Profile dropdown -> "Preferences"
  <- Shortcut: Ctrl+,

Help Modal
  <- Topbar "Help" menu -> "Manual" (other Help items open URLs)
  <- Shortcut: F1

Profile/Account Modal
  <- Topbar right -> user avatar button
  <- Statusbar right -> user chip / name
```

---

## 6. MODAL OVERLAY RENDERING

```
App Root Panel ("app")
  +-- Topbar
  +-- Work area (editors)
  +-- Statusbar
  |
  +-- [OVERLAY when modal is open]
       +-- Backdrop div (absolute, inset-0, bg semi-transparent)
       +-- Modal Panel (centered, mt:80px, shadow, rounded)
```

GPUI implementation:
- Backdrop: `div().absolute().inset_0().bg(hsla(0,0,0,0.5))`
- Modal: `div().absolute().mx_auto().mt(px(80.)).w(px(modal_width)).rounded(px(8.)).shadow_lg()`

---

## 7. WELCOME WINDOW (BModalType::Splash)

See [spaces/modals/welcome.md](spaces/modals/welcome.md). Layout is Blender's
`WM_MT_splash`: banner (Phiber + version) → split (New File | Recent Files ×5
+ More…, or Getting Started links when there are no recents) → split (Open…,
Recover Last Session | What's New, Support Phiber). Every block is one rounded
`BPanelRole::Card`.

## 8. FILE BROWSER (BModalType::FileBrowser)

See [spaces/modals/file_browser.md](spaces/modals/file_browser.md). The modal
and the `File` / `Asset` editors share one implementation
(`part/custom/file/{header,path_bar,sidebar,execute}.rs`).

## 9. ABOUT (BModalType::About)

`modal/about` — Blender's `WM_MT_splash_about`: a facts card (Version, Date,
Hash, Branch, Windowing Environment — build values come from
`PHIBER_BUILD_DATE` / `PHIBER_BUILD_HASH` / `PHIBER_BUILD_BRANCH` when set),
the Phiber icon card, and a footer of links (Release Notes, Credits, License,
Phiber Website, Donate) from `modal/about/set.rs`.

## 10. RENDER WINDOW (BModalType::Render)

`modal/render` — Blender opens F12 renders in a separate window that hosts an
Image Editor showing `Render Result` (`render_view_open`,
`render_display_type == 'WINDOW'`). Phiber builds `BHostItem(Image)` inside a
window-sized modal card (no inner padding, like the File Browser). Actual
rendering into it is not implemented yet.
