# gpui_glass

A surface made from what is drawn beneath it, after Apple's Liquid Glass:
blurred, bent at the rim, tinted and lit. This crate is the one home of
glass: every renderer draws from its plan. It has no dependencies and
builds for wasm32 as it is. It lives with the renderers, in `appkit/gpu`.

| Folder | Holds |
|---|---|
| `src/material.rs`, `data/style.tsv` | `GlassStyle`, `GlassMaterial`; each style's numbers are a row of the table |
| `src/effect.rs` | `GlassEffect`, `GlassContainer`: Apple's `NSGlassEffectView` names, as data |
| `src/shape.rs` | `GlassShape`, `GlassBounds`: the outline as a distance field |
| `src/cpu/` | the stages on the CPU, the reference every GPU backend is checked against |
| `src/plan.rs` | `GlassPlan`: the one copy and four draws a GPU renderer makes, the same on every backend |
| `src/glass.hlsl`, `src/glass.wgsl` | the stages as shaders, for DirectX and for wgpu and the web |
| `../gpui_windows/src/directx_backdrop.rs` | the DirectX pass: the renderer's own, drawing from this crate's `GlassPlan` |
| `src/tests/` | the tests, apart from the code and in the same folders |
| `examples/plan.rs` | prints a plan as JSON, for a renderer across a language boundary |

```
cargo test -p gpui_glass
cargo test -p gpui_windows backdrop
cargo check -p gpui_glass --target wasm32-unknown-unknown
cargo run -p gpui_glass --example plan -- 800 600
```

Start at [ARCHITECTURE.md](ARCHITECTURE.md); what moved and the steps are
in [PLAN.md](PLAN.md); what is left, in order, is in [TODO.md](TODO.md); what each part must do, with Apple's header lines, is in
[SPECS.md](SPECS.md); where Apple's interfaces and the other glass code on
disk are is in [reference/README.md](reference/README.md).

Glass goes on windows only: panes, floating panels, a 3D view's strips.
Buttons, tiles and icons stay opaque. No colour is fixed here: tint, edge
and shadow come from the caller's theme.
