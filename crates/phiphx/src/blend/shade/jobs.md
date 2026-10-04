# Jobs — `phiphx/blend/shade`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 2 | 7 |

## Todo

- [ ] Per-pixel procedural textures (Noise, Voronoi) and Bump; a Mix Shader's two closures as two lobes in the kernel (folded to one for now); B-spline / cardinal / HSV colour ramps
- [ ] Image colour space and UDIM tiles

## Done

- [x] The surface from the Material Output (`surface.rs`): the active output's Surface link, through Mix / Add Shader and reroutes, to the closures — Principled, Emission, Glass / Refraction, Transparent / Holdout, Diffuse / Translucent, Glossy, Sheen, Subsurface — folded to one `BlendBsdf` (Mix lerps by its factor, Add averages and sums emission); Light Path folds for a camera ray; the Principled the surface reaches is the one whose textures and normal map are read (`tests/surface_truth.rs`)

- [x] Material slots (object and mesh), viewport colour, Principled base colour
- [x] Image keys and packed image bytes
- [x] `BlendBsdf`: Principled Base Color, Roughness, Metallic, Alpha, IOR, Transmission, Emission (+ 3.x names) from sockets — 148 materials match Blender 5.0.1 (`tests/bsdf_truth.rs`)
- [x] Node links: Principled `Normal` ← Normal Map ← Image Texture (`BlendNormalMap`, strength) (`tests/normal_truth.rs`, NORMAL-MAP fixture)
- [x] Base Color texture = first Image Texture upstream of Principled Base Color (through Mix / Color Ramp / Hue-Sat …, not Bump / Normal Map); 150 materials across fixtures, Mannequins and LAND match Blender 5.0.1 (`tests/base_image_truth.rs`)
- [x] Colour graph folding (`eval.rs`, `blend.rs`): Mix (12 blend modes, clamps), legacy MixRGB, Color Ramp, Hue/Sat, Invert, RGB/Value, Reroute → one value per Principled input (image = white tint, procedural = mean); 150 materials match Blender 5.0.1 with ramps evaluated by Blender (`tests/bsdf_truth.rs`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
