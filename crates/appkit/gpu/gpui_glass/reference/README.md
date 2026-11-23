# reference

Everything glass on disk outside this crate, by path (from `gemphi/`).
None of it is built here. Read 2026-11-10.

## Apple's declarations

- `phi/apple-source/ruskit/AppKit.framework/Versions/C/Headers/NSGlassEffectView.h`
  (60 lines, macOS 26): `NSGlassEffectViewStyle` regular and clear (:14-19);
  `NSGlassEffectView` contentView, cornerRadius, tintColor, style (:23-38);
  `NSGlassEffectContainerView` contentView, spacing (:43-57).
- Same folder, `NSVisualEffectView.h` (120 lines): 14 materials and 5
  deprecated ones (:19-70), blending modes behindWindow and withinWindow
  (:72-78), states (:80-89), maskImage, emphasized.
- Same folder, `NSBackgroundExtensionView.h` (45 lines, macOS 26).
- `phi/apple-source/ruskit/SwiftUI.framework/Versions/A/Modules/SwiftUI.swiftmodule/arm64e-apple-macos.swiftinterface`:
  `GlassButtonStyle` (:1247), `GlassProminentButtonStyle` (:3378),
  `backgroundExtensionEffect` (:12093).
- `phi/apple-source/ruskit/Interfaces/MacOSX/QuartzCore/Headers/CALayer.h`:
  `filters`, `backgroundFilters`, `compositingFilter` (:660-670).
- `phi/apple-source/ruskit/MetalPerformanceShaders.framework/Versions/A/Frameworks/MPSImage.framework/Versions/A/Headers/MPSImageConvolution.h`:
  `MPSImageBox` (:146), `MPSImageTent` (:219), `MPSImageGaussianBlur` (:237).

Not on disk: all of UIKit (`UIGlassEffect`, `UIVisualEffectView`), SwiftUI's
`Glass`, `glassEffect` and `Material`, `CABackdropLayer`, Quick Look.

## The Apple-style code in the workspace

`ext/phiber-refs/apple/src`, with a byte-identical copy at
`phikit/crates/apple/src`:

- `appkit/gpu/glass_pipeline/` (5 files) and `appkit/gpu/shaders/glass_blur.wgsl`,
  `glass_composite.wgsl`: a compute Gaussian both ways, then saturation, tint
  and opacity. No capture, shrink, lens, rim or shape.
- `appkit/core/render/effects.rs:84-151`: `fill_glass`, a box blur in place.
- `swiftui/styling/material.rs`, `uikit/components/visual_effect_view.rs`,
  `appkit/theme/appearance.rs`: the tables of Apple's material names. The
  thickness numbers in `../data/style.tsv` come from the first.

## The older copy of this crate

`phi/crates/appkit/src/glass` and `phi/crates/appkit/tests/glass.rs`: what
this crate was before 2026-11-10, with the numbers still inline. Superseded.

## Raven

`ext/phiber-refs/Raven` (C, Vulkan): glass inside a window, in one frame.

- `src/render/draw.c:256-303`: captures merged by dirty rectangle, at most 24 a frame.
- `src/render/vulkan/vk_frame.c:129-133`: the pass ends at each backdrop, the capture runs, the pass resumes.
- `src/render/vulkan/vk_glass.c:234-279`: reduce to an eighth in three steps, enlarge twice.
- `src/render/vulkan/shaders/blur.frag`: dual filtering.
- `src/render/vulkan/shaders/ui.frag:65-120`: rim bend, refraction, dispersion, frost, saturation, tint, rim light.
- `docs/ARCHITECTURE.md:147-202`.

## In phikit, outside this folder

- `crates/appkit/gpu/src/scene.rs:823-852`: `gpui::BackdropStyle`, `Backdrop`, the primitive.
- `crates/appkit/gpu/gpui_windows/src/directx_renderer.rs:742`: where DirectX draws them.
- `crates/appkit/gpu/gpui_wgpu/src/wgpu_renderer.rs:1307`: where wgpu skips them.
- `crates/appkit/src/glass_view.rs`: the `NSView` glue.
- `crates/phiber/src/theme/style/glass.rs`: phiber's glass mode and its numbers.
- `crates/phiber/src/view/draw/flex/paint.rs:48`: `frosted`, which decides what is glass.
- `crates/phiray/src/kernels/image.rs`: `Convolve`, a compute convolution over buffers, not textures.
