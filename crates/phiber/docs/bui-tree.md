# BUI Tree Architecture, Theme Palette & Activity Bar Specification

## 1. Physical & Logical Hierarchy Blueprint

The BUI architecture follows a strict 8-tier hierarchy from root window to atomic card items:

$$\text{Space} \longrightarrow \text{Zone} \longrightarrow \text{Area} \longrightarrow \text{Host} \longrightarrow \text{Region} \longrightarrow \text{Pane} \longrightarrow \text{Part} \longrightarrow \mathbf{Card} (\text{Header} \mid \text{Body} \mid \text{Footer})$$

```
crates/phiber/src/bui/
├── mod.rs                  # BUI root re-exports (BView, BState, BKey, BPalette, BAction)
├── card.rs                 # Universal Collapsible Card (Header, Body, Footer, List, Item)
├── bview.rs                # Declarative BView Node Enum & .with_* Fluent Methods
├── bview_debug.rs          # Visual tree layout inspector
├── render.rs               # GPUI into_elm & into_popup_item rendering engine
├── space.rs                # SpaceView (Top, Left Activity, Center, Right, Bottom Zone Coordinator)
├── area.rs                 # AreaView (Multi-host container & splitter coordinator)
├── host.rs                 # HostView (Active Editor Container: View3D, Outliner, Buts, etc.)
├── region.rs               # RegionView (Header, Center, Footer region coordinator)
├── pane.rs                 # PaneView (Left, Center, Right pane coordinator)
├── modal.rs                # ModalView (Floating Dialogs: Settings, Social Themes, File Browser)
├── theme.rs                # Theme Palettes, metrics, fills, and sheen gradients
├── icon.rs                 # Vector icon registry & rendering
│
└── part/                   # 11 Modular Parts (Flat & Standardized)
    ├── mod.rs              # PartView dispatcher matching BPartType
    ├── activity.rs         # ActivityView (Left Activity Bar: Top, Center Stream, Bottom Profile)
    ├── channels.rs         # ChannelsView (Outliner collections / animation channels)
    ├── custom.rs           # CustomView (3D Viewport canvas container)
    ├── menubar.rs          # MenubarView (Top-level & editor header menus)
    ├── modebar.rs          # ModebarView (Interaction mode selector & tool settings)
    ├── panel.rs            # PanelView (N-Panel Property Accordions & Tabs)
    ├── sidebar.rs          # SidebarView (N-Panel vertical icon tabs & drawer)
    ├── spacebar.rs         # SpacebarView (Workspace tabs + Modal trigger buttons)
    ├── statusbar.rs        # StatusbarView (Footer breadcrumbs & scene polycount stats)
    ├── switcher.rs         # SwitcherView (2-Column Editor Host & Mode Picker)
    └── toolbar.rs          # ToolbarView (Left tool icon column: Select, Move, Rotate, Scale)
```

---

## 2. Complete Color Form Matrix & Token Registry

Every UI component draws from this mathematical color palette with support for gradient sheen overlays, translucent glass backdrops, and active status indicators:

| Token Name | Role / Purpose | Base HSLA (Blender Dark) | Sheen Top HSLA | Sheen Bottom HSLA | Border / Stroke |
|---|---|---|---|---|---|
| `bg_app` | Window root background | `hsla(220°, 13%, 10%, 1.0)` | `hsla(220°, 13%, 12%, 1.0)` | `hsla(220°, 13%, 10%, 1.0)` | `hsla(220°, 10%, 8%, 1.0)` |
| `bg_activity` | Activity bar (Leftmost) | `hsla(220°, 15%, 12%, 1.0)` | `hsla(220°, 15%, 14%, 1.0)` | `hsla(220°, 15%, 11%, 1.0)` | `hsla(220°, 10%, 16%, 1.0)` |
| `bg_activity_sec` | Activity 2nd column drawer | `hsla(220°, 14%, 13%, 1.0)` | `hsla(220°, 14%, 15%, 1.0)` | `hsla(220°, 14%, 12%, 1.0)` | `hsla(220°, 10%, 16%, 1.0)` |
| `bg_top` | Workspace top header bar | `hsla(220°, 12%, 18%, 1.0)` | `hsla(220°, 12%, 20%, 1.0)` | `hsla(220°, 12%, 17%, 1.0)` | `hsla(220°, 10%, 14%, 1.0)` |
| `bg_card` | Universal Card background | `hsla(220°, 12%, 16%, 1.0)` | `hsla(220°, 12%, 18%, 1.0)` | `hsla(220°, 12%, 15%, 1.0)` | `hsla(220°, 10%, 13%, 1.0)` |
| `bg_card_header` | Card header titlebar | `hsla(220°, 12%, 19%, 1.0)` | `hsla(220°, 12%, 21%, 1.0)` | `hsla(220°, 12%, 18%, 1.0)` | `hsla(220°, 10%, 14%, 1.0)` |
| `bg_card_footer` | Card footer status bar | `hsla(220°, 12%, 14%, 1.0)` | `hsla(220°, 12%, 15%, 1.0)` | `hsla(220°, 12%, 13%, 1.0)` | `hsla(220°, 10%, 12%, 1.0)` |
| `bg_header` | Editor area headers | `hsla(220°, 12%, 18%, 1.0)` | `hsla(220°, 12%, 20%, 1.0)` | `hsla(220°, 12%, 17%, 1.0)` | `hsla(220°, 10%, 13%, 1.0)` |
| `bg_viewport` | 3D Canvas / Node background | `hsla(220°, 10%, 20%, 1.0)` | `hsla(220°, 10%, 22%, 1.0)` | `hsla(220°, 10%, 19%, 1.0)` | `hsla(220°, 10%, 15%, 1.0)` |
| `bg_status` | Bottom status bar | `hsla(220°, 12%, 13%, 1.0)` | `hsla(220°, 12%, 14%, 1.0)` | `hsla(220°, 12%, 12%, 1.0)` | `hsla(220°, 10%, 11%, 1.0)` |
| `bg_hover` | Interactive item hover | `hsla(220°, 10%, 24%, 1.0)` | `hsla(220°, 10%, 26%, 1.0)` | `hsla(220°, 10%, 23%, 1.0)` | `hsla(28°, 95%, 54%, 0.6)` |
| `bg_active` | Selected tab / active item | `hsla(220°, 10%, 28%, 1.0)` | `hsla(220°, 10%, 30%, 1.0)` | `hsla(220°, 10%, 26%, 1.0)` | `hsla(28°, 95%, 54%, 1.0)` |
| `bg_icon` | Icon button resting | `hsla(220°, 10%, 18%, 1.0)` | `hsla(220°, 10%, 20%, 1.0)` | `hsla(220°, 10%, 17%, 1.0)` | `hsla(220°, 10%, 14%, 1.0)` |
| `bg_icon_hover` | Icon button hover | `hsla(220°, 10%, 26%, 1.0)` | `hsla(220°, 10%, 28%, 1.0)` | `hsla(220°, 10%, 24%, 1.0)` | `hsla(28°, 95%, 54%, 0.8)` |
| `bg_icon_active`| Icon button pressed | `hsla(220°, 10%, 30%, 1.0)` | `hsla(220°, 10%, 32%, 1.0)` | `hsla(220°, 10%, 28%, 1.0)` | `hsla(28°, 95%, 54%, 1.0)` |
| `bg_modal_dim` | Fullscreen modal overlay | `hsla(0°, 0%, 0%, 0.65)` | `hsla(0°, 0%, 0%, 0.65)` | `hsla(0°, 0%, 0%, 0.65)` | `transparent` |
| `text` | Primary high-contrast text | `hsla(0°, 0%, 92%, 1.0)` | — | — | — |
| `text_dim` | Secondary text & labels | `hsla(0°, 0%, 55%, 1.0)` | — | — | — |
| `accent` | Brand orange / focus highlight | `hsla(28°, 95%, 54%, 1.0)` | `hsla(28°, 95%, 58%, 1.0)` | `hsla(28°, 95%, 50%, 1.0)` | `hsla(28°, 95%, 65%, 1.0)` |
| `accent_blue` | Discord / Chat online badge | `hsla(217°, 91%, 60%, 1.0)` | `hsla(217°, 91%, 64%, 1.0)` | `hsla(217°, 91%, 56%, 1.0)` | `hsla(217°, 91%, 70%, 1.0)` |
| `accent_spark` | AI Stream spark gold | `hsla(45°, 100%, 51%, 1.0)` | `hsla(45°, 100%, 58%, 1.0)` | `hsla(45°, 100%, 45%, 1.0)` | `hsla(45°, 100%, 65%, 1.0)` |
| `online_green` | Live user presence | `hsla(142°, 70%, 45%, 1.0)` | — | — | — |
| `border` | Subtle structural dividing line | `hsla(220°, 10%, 12%, 1.0)` | — | — | — |

---

## 3. The Activity Bar Specification

The **Activity Bar** occupies the extreme left of the application at **full screen height**, using a fixed `44px` toolbar width that expands to show branding, spaces, live streams, social chat, and settings.

### 3.1 Vertical Three-Pane Layout (`justify-between`)

```
┌────────────────────────┐  ◄── Brand Header (Default "P" logo -> Hover toggle -> "PHIBER" on expand)
│  [P] PHIBER       [◀]  │
├────────────────────────┤
│  [⌂] Home (Layout)     │  ◄── TOP SECTION
│  [💬] Chat (Discord)   │      - Home: Switches to Layout space & closes 2nd column
│  [☷] Spaces (Workspaces│      - Chat: Slides out 2nd column with users & channels
├────────────────────────┤      - Spaces: Slides out 2nd column with workspace list
│                        │
│  [✨] Spark (Stream)   │  ◄── CENTER SECTION (Live collaborative stream / AI prompt)
│                        │
├────────────────────────┤
│  [⚙] Settings (Themes) │  ◄── BOTTOM SECTION
│  [👤] Profile (Avatar) │      - Settings: Opens Zed-style Theme & Social modal
└────────────────────────┘      - Profile: User status, discord ID, account info
```

### 3.2 Two-Column Expansion & Discord-Style Social Drawer

When clicking a primary icon (e.g. `Chat` or `Spaces`):
1. **Primary Column (`44px`)**: Keeps high-contrast active indicator on the selected tab.
2. **Secondary Column Drawer (`220px`)**: Slides out smoothly with the secondary panel:
   - **Chat View**:
     - Direct Messages header with unread badges.
     - User rows with Discord-style avatars, male/female gender icons, usernames (`@alice`, `@bob`, `@carol`), and green online status indicators.
     - Voice & Viewport Sharing channels (`#viewport-collab`, `#general-3d`).
   - **Spaces View**:
     - Workspace list in secondary palette (`Layout`, `Modeling`, `Sculpting`, `UV Editing`, `Texture Paint`, `Shading`, `Animation`, `Compositing`, `Geometry Nodes`, `Scripting`).
3. **Clicking `Home`**: Immediately navigates to `Layout` space and closes the secondary column.

---

## 4. Zed-Inspired Theme & Social Settings Specification

The **Theme Settings Modal** (`BModalType::Settings`) delivers a modern, social-enabled customization hub:

### 4.1 Feature Grid

| Feature Area | Capabilities |
|---|---|
| **Live Theme Switcher** | Instant preview of curated palettes: *Blender Dark, Apple Dark, Tokyo Night, Nord, Gruvbox, Cyberpunk, Solarized Dark, GitHub Dark*. |
| **Community Marketplace** | Browse trending community-created themes with real-time star ratings, download counters, and author badges. |
| **Creator Profiles** | View theme creators, their published palettes, and social links. |
| **Live Collaborative Sync** | Sync active theme across team sessions in real time. |
| **Social Comments & Reviews**| Leave feedback and rate theme palettes directly in the app. |
| **Custom Palette Editor** | Fine-tune HSL tokens (`bg_app`, `accent`, `sheen`, `radii`) with instant live re-rendering. |
