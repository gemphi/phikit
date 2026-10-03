# phigpu — the window and GPU layer (vendored)

`gpui` is the framework phiber's whole interface runs on: it owns the window,
input, layout and the GPU drawing. phiber depends on `gpui` and `gpui_platform`
directly; everything else here is reached through them.

| Crate | What it does |
|---|---|
| `gpui` | the framework: windows, elements, layout, text, drawing |
| `gpui_platform` | picks the platform backend below at build time |
| `gpui_windows` | the Windows backend (what you run) |
| `gpui_macos`, `gpui_linux`, `gpui_web` | the other platforms' backends |
| `gpui_wgpu` | the wgpu renderer `gpui_linux` and `gpui_web` draw with |
| `gpui_macros`, `gpui_shared_string`, `gpui_util` | small parts `gpui` itself needs |

Vendored — we don't design these. Widgets live in [`../phiui`](../phiui),
support crates in [`../phicore`](../phicore).
