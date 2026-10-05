# Jobs — `phiphx/blend/shade`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 2 | 10 |

## Todo

- [ ] Sky / Gabor / White Noise textures, 4D (W) textures, Mapping's Rotation and Texture / Vector / Normal types, Generated / Object / UV only (no Normal, Camera, Window, Reflection), one texture per socket (a Mix node between texture and socket folds to its mean); B-spline / cardinal / HSV colour ramps
- [ ] Image colour space and UDIM tiles

## Done

- [x] Every other texture node (`texnode.rs`): Noise's five types with Offset / Gain and Normalize, Voronoi's features (F1, F2, Smooth F1, Distance to Edge, N-Sphere Radius) and metrics, Checker, Brick (offset / squash and frequencies), Gradient's types, Magic's depth, Wave's type / directions / profile — sockets and storage with `node_shader_tex_*.cc`'s defaults (`tests/texture_nodes_truth.rs`)
- [x] Per-pixel procedural textures (`procedural.rs`, `BlendProcedural`): Principled Base Color ← [Color Ramp ←] Noise / Voronoi Texture, the surface Mix Shader's Fac the same way, Principled Normal ← Bump(Strength, Distance).Height ← [ramp ←] texture — the texture's inputs, its coordinates (Generated / Object / UV through a Point Mapping's Location and Scale) and the ramp baked to 256 linear RGBA stops, for the tracer to evaluate per pixel (`phiray`'s `procedural.wgsl`); sockets matched by identifier or name (Blender 4.x's "Factor" labels); a Normal Map behind a Bump is still read (`tests/procedural_truth.rs`, PROCEDURAL fixture)
- [x] A Mix Shader's two closures as two lobes (`BlendMix`, `surface_mix`): each side folded, the factor, and which side holds the reached Principled — the tracer shades both (`tests/surface_truth.rs`)
- [x] The surface from the Material Output (`surface.rs`): the active output's Surface link, through Mix / Add Shader and reroutes, to the closures — Principled, Emission, Glass / Refraction, Transparent / Holdout, Diffuse / Translucent, Glossy, Sheen, Subsurface — folded to one `BlendBsdf` (Mix lerps by its factor, Add averages and sums emission); Light Path folds for a camera ray; the Principled the surface reaches is the one whose textures and normal map are read (`tests/surface_truth.rs`)

- [x] Material slots (object and mesh), viewport colour, Principled base colour
- [x] Image keys and packed image bytes
- [x] `BlendBsdf`: Principled Base Color, Roughness, Metallic, Alpha, IOR, Transmission, Emission (+ 3.x names) from sockets — 148 materials match Blender 5.0.1 (`tests/bsdf_truth.rs`)
- [x] Node links: Principled `Normal` ← Normal Map ← Image Texture (`BlendNormalMap`, strength) (`tests/normal_truth.rs`, NORMAL-MAP fixture)
- [x] Base Color texture = first Image Texture upstream of Principled Base Color (through Mix / Color Ramp / Hue-Sat …, not Bump / Normal Map); 150 materials across fixtures, Mannequins and LAND match Blender 5.0.1 (`tests/base_image_truth.rs`)
- [x] Colour graph folding (`eval.rs`, `blend.rs`): Mix (12 blend modes, clamps), legacy MixRGB, Color Ramp, Hue/Sat, Invert, RGB/Value, Reroute → one value per Principled input (image = white tint, procedural = mean); 150 materials match Blender 5.0.1 with ramps evaluated by Blender (`tests/bsdf_truth.rs`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
