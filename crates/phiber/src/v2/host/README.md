# v2/host

Host — one editor kind. `host/<name>/` folders own region declarations only.
Widget content lives in `part::custom::<name>`.

## Files

| File | What it is |
|---|---|
| `action/` | Dope Sheet host. Blender ref: `space_action/space_action.cc:987-`. Regions: WINDOW, HEADER, FOOTER, CHANNELS, UI. |
| `activity/` | Activity host — Phiber-native global navigation rail. Region declarations only; the icon-strip content lives in `part::custom::activity` via the… |
| `asset/` | Asset Browser host — shares Blender's `space_file` in asset mode. Regions: HEADER (menus) + WINDOW. The TOOLS (library / catalogs) and UI (path… |
| `buts/` | Properties (Buts) host — region declarations only. Widget content lives in `part::custom::buts`. Blender ref:… |
| `clip/` | Movie Clip Editor host. Blender ref: `space_clip/space_clip.cc:1256-`. Regions: WINDOW, PREVIEW, UI, TOOLS, HEADER, TOOL_HEADER, HUD. |
| `compositor/` | Compositor host — shares Blender's `space_node` implementation. Same regions as Node: WINDOW, HEADER, ASSET_SHELF+HEADER, UI, TOOLS. |
| `console/` | Python Console host. Blender ref: `space_console/space_console.cc:370-`. Regions: WINDOW, HEADER. |
| `drivers/` | Drivers Editor host — a mode of Blender's `space_graph` (`SIPO_MODE_DRIVERS`). Regions match Graph Editor: WINDOW, HEADER, FOOTER, CHANNELS, UI. |
| `file/` | File Browser host. Blender ref: `space_file/space_file.cc:1043-`. Regions: HEADER (menus) + WINDOW. The Blender TOOLS (bookmarks) and UI (path… |
| `geonode/` | Geometry Nodes Editor host — shares Blender's `space_node` implementation. |
| `image/` | Image Editor host. Blender ref: `space_image/space_image.cc:1343-`. Regions: WINDOW, UI, TOOLS, TOOL_HEADER, HEADER, ASSET_SHELF, ASSET_SHELF_HEADER. |
| `info/` | Info (log) host. Blender ref: `space_info/space_info.cc:271-`. Regions: WINDOW, HEADER. |
| `ipo/` | Graph Editor host (`Ipo` = interpolation, historical name). Blender ref: `space_graph/space_graph.cc:1009-`. Regions: WINDOW, HEADER, FOOTER,… |
| `item.rs` | `BHostItem` — one host. |
| `list.rs` | `BHostList` — iterate a set of hosts (Switcher popover contents). |
| `logic/` | Logic Editor host (legacy). Blender 5.2 dropped the Game Engine and the Logic Editor with it; retained here for `BHostType` completeness only.… |
| `mod.rs` | Host — one editor kind. |
| `nla/` | NLA host. Blender ref: `space_nla/space_nla.cc:680-`. Regions: WINDOW, HEADER, FOOTER, CHANNELS, UI. |
| `node/` | Node Editor host (generic). Blender ref: `space_node/space_node.cc:1745-`. Regions: WINDOW, HEADER, ASSET_SHELF, ASSET_SHELF_HEADER, UI, TOOLS. |
| `outliner/` | Outliner host — region declarations only. Widget content lives in `part::custom::outliner`. Blender ref: `editors/space_outliner/space_outliner.cc`. |
| `prefs/` | Preferences host. Blender ref: `space_userpref/space_userpref.cc:419-`. Regions: WINDOW, HEADER, UI (NAV_BAR wrapping), EXECUTE. |
| `seq/` | Video Sequencer host. Blender ref: `space_sequencer/space_sequencer.cc:1210-`. Regions: WINDOW, PREVIEW, UI, TOOLS, CHANNELS, TOOL_HEADER, HEADER,… |
| `set.rs` | Host set — resolves `BHostType` → `&'static BHostSpec`. |
| `shader/` | Shader Editor host — shares Blender's `space_node` implementation. |
| `spec.rs` | `BHostSpec` — one editor host with its regions inline. |
| `spreadsheet/` | Spreadsheet host. Blender ref: `space_spreadsheet/space_spreadsheet.cc:795-`. Regions: WINDOW, HEADER, FOOTER, UI, TOOLS (dataset selector). |
| `statusbar/` | Statusbar host — region declarations only. No center content. Blender ref: DNA `SPACE_STATUSBAR` (single HEADER region, global screen area). |
| `text/` | Text Editor host. Blender ref: `space_text/space_text.cc:470-`. Regions: WINDOW, UI, HEADER, FOOTER. |
| `time/` | Timeline host — region declarations only. Widget content lives in `part::custom::time`. Blender ref: `editors/space_action/space_action.cc` in… |
| `topbar/` | Topbar host — menu, workspace tabs, zone visibility, and host switcher. Blender ref: DNA `SPACE_TOPBAR` (single HEADER region, global screen area). |
| `type.rs` | `BHostType` — editor kind. |
| `uv/` | UV Editor host — reuses Blender's `space_image` implementation in UV mode. Same regions as Image: WINDOW, UI, TOOLS, TOOL_HEADER, HEADER,… |
| `view3d/` | View3D host — region declarations only. Widget content lives in `part::custom::view3d`. Blender ref: `editors/space_view3d/space_view3d.cc`. |

Status and next steps: [`jobs.md`](jobs.md).
