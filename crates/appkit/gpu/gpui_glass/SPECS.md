# glass — specs

What each part must do. One row is one requirement, with the Apple
declaration it answers, where there is one. "Today" says whether it is met.

## Sources

| Short | What | Where (from `gemphi/`) |
|---|---|---|
| **GV** | `NSGlassEffectView`, `NSGlassEffectContainerView` | `phi/apple-source/ruskit/AppKit.framework/Versions/C/Headers/NSGlassEffectView.h` |
| **VE** | `NSVisualEffectView`: materials, blending modes, states | same folder, `NSVisualEffectView.h` |
| **BE** | `NSBackgroundExtensionView` | same folder, `NSBackgroundExtensionView.h` |
| **CA** | `CALayer.backgroundFilters` | `phi/apple-source/ruskit/Interfaces/MacOSX/QuartzCore/Headers/CALayer.h` |
| **MPS** | `MPSImageGaussianBlur`, `MPSImageBox` | `phi/apple-source/ruskit/MetalPerformanceShaders.framework/Versions/A/Frameworks/MPSImage.framework/Versions/A/Headers/MPSImageConvolution.h` |
| **RAV** | Raven: glass in a Vulkan GUI library | `ext/phiber-refs/Raven` |
| index | every glass file on disk, and what each holds | [reference/README.md](reference/README.md) |

Apple declares names and properties, not numbers or shader stages. Where a
row states behaviour Apple does not write down, the row says "ours".

## What a host sets

| Part | Must | Apple | Today |
|---|---|---|---|
| `GlassStyle` | be `Regular` or `Clear` | `NSGlassEffectViewStyle` GV:14-19 | met |
| `GlassEffect` | carry a style, a corner radius and an optional tint | `style`, `cornerRadius`, `tintColor` GV:31-37 | met |
| `GlassContainer` | merge the glass inside it within `spacing`, as one layer | `spacing` GV:56 | met on the CPU; a GPU backend draws each piece alone |
| material by name | give a material for Apple's thickness names | VE:19-70 (the names; the numbers are ours) | met: `GlassMaterial::named` |
| colours | take tint, edge and shadow from the caller's theme; fix none | - | met |
| numbers | keep each style's numbers in `data/style.tsv`, not in code | - | met for the material; the shader's own constants are still in the shader |
| blending mode | blur what is inside the window; the desktop behind it is the system's job | `NSVisualEffectBlendingMode` VE:72-78 | within-window here; behind-window is gpui's `WindowBackgroundAppearance` |
| state | go solid when the user asks for less transparency | `NSVisualEffectState` VE:80-89 | the host's choice (phiber: Reduce Transparency) |

## The stages

| Part | Must | Apple | Today |
|---|---|---|---|
| capture | read only what is already drawn under the glass, with room for the blur and the lens (ours) | `backgroundFilters` CA:670 | met |
| no feedback | never read the target being written; RAV ends its pass, captures, and resumes (`src/render/vulkan/vk_frame.c:129-133`) | - | met on DirectX (a copy); the wgpu hook must do the same |
| shrink | blur a half or a quarter-size copy: a wide blur loses nothing there | - | met |
| blur | be close to a Gaussian of sigma = half the blur radius | `MPSImageGaussianBlur` sigma MPS:251-275 | CPU: three box passes; GPU: a true Gaussian, both ways |
| lens | read from further inside near the rim (ours) | - | met |
| saturation | bring back the colour a blur greys out | - | met |
| tint | thicken over what stands far from the tint's lightness, so text holds 4.5:1 without veiling the rest (ours) | - | met |
| edge light | light the rim where it faces the light (ours) | - | met |
| shadow | fall below the glass and fade | - | CPU only |
| content | leave what is drawn after the glass sharp, on top | `contentView` GV:28 | met |

## The plan and the backends

| Part | Must | Today |
|---|---|---|
| `GlassPlan` | give every GPU backend the same copy and the same four blocks for one request | met |
| `GlassParam` | be 144 bytes, laid out the same in HLSL and WGSL | met, tested |
| `src/glass.wgsl` | be the stages of `glass.hlsl`, body for body | met; compiled and run on WebGPU with a plan from the crate |
| DirectX | draw from the crate's plan | met: `gpui_windows/src/directx_backdrop.rs` makes a `GlassRequest` and draws the plan's four stages; three tests draw on a real device |
| wgpu and the web | draw `Backdrops` | no: `gpui_wgpu` skips them (`wgpu_renderer.rs:1307`) |
| GPU against CPU | match the reference within a tolerance, in a test | no test yet |
| wasm | build for `wasm32-unknown-unknown` with no feature off | met |

## Not taken

A second, lesser pipeline: the wgpu compute blur and composite under
`ext/phiber-refs/apple/src/appkit/gpu/glass_pipeline` has no capture, no
shrink, no lens, no rim and no shape, so its stages are a subset of this
crate's. Its numbers for Apple's material names were taken; its code was
not. Raven's dispersion and outside reflection are not in this glass.
