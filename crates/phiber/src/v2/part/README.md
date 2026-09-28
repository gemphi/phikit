# v2/part

Part — the leaf. Every variant lives in its own subfolder with item + list.

## Files

| File | What it is |
|---|---|
| `activity/` | Activity Bar — Phiber-native leftmost vertical navigation. |
| `assetshelf/` | Asset Shelf — brush / preset / texture thumbnails. Blender ref: `RGN_TYPE_ASSET_SHELF`. |
| `caption/` | Window caption — Phiber draws its own title bar in the theme colours (the native one is hidden), so the Topbar carries the platform window… |
| `channels/` | Channels — vertical list of animation channels. |
| `custom/` | Custom — reroutes to the active host's center content. |
| `execute/` | Execute bar — footer action buttons (Cancel / Open / Save …). Blender ref: `RGN_TYPE_EXECUTE`. |
| `hud/` | HUD — floating overlay on top of a viewport canvas. Blender ref: `RGN_TYPE_HUD`. |
| `item.rs` | `BPartItem` — one part; dispatcher by `BPartType`. |
| `list.rs` | `BPartList` — iterate the parts of a pane. |
| `menubar/` | Menubar — horizontal row of menu buttons. Layout is driven by the host's BPaneSpec; the Switcher part separately provides the editor-type icon on… |
| `mod.rs` | Part — the leaf. |
| `modebar/` | Modebar — the top-left mode dropdown of a host's header. Blender ref: `VIEW3D_HT_header` mode picker (`layout.template_header()` + `object.mode`… |
| `navbar/` | NavBar — vertical column of context tabs (Properties). |
| `panel/` | Panel — accordion of property sections with RNA-backed props. |
| `set.rs` | Part set — one `BPartSpec` per `BPartType`, order matches the enum. |
| `sidebar/` | Sidebar — vertical tab strip. |
| `spacebar/` | Spacebar — workspace tabs at the top center. |
| `spec.rs` | `BPartSpec` — one part descriptor (identity + label). |
| `statusbar/` | Statusbar (part) — horizontal segments: scene • frame • stats • mem • version. |
| `strip.rs` | Shared vertical icon-strip state helpers. |
| `switcher/` | Switcher — the editor-type dropdown in a host header (Blender's `Editor Type` selector at the far left of every header). The trigger shows the… |
| `toolbar/` | Toolbar — vertical column of tool icons. |
| `toolprops/` | Tool Properties — settings shown alongside the mode bar for the active tool. Blender ref: `RGN_TYPE_TOOL_HEADER` content. |
| `type.rs` | `BPartType` — leaf variants. |
| `zonebar/` | Topbar zone-visibility controls for zones declared by the active space. |

Status and next steps: [`jobs.md`](jobs.md).
